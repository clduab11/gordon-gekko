"""
Test suite for SecurityScanner component using TDD methodology.

This module provides comprehensive test coverage for the security scanner,
focusing on vulnerability detection, threat monitoring, and security validation.
"""

import pytest
import asyncio
from unittest.mock import Mock, patch, AsyncMock
from typing import Dict, Any, List

from src.gordon_gekko.deployment.security_scanner import (
    SecurityScanner,
    Vulnerability,
    ThreatAlert,
    SecurityViolation,
    ScanResult,
)


class TestSecurityScannerInitialization:
    """Test SecurityScanner initialization and basic setup."""

    def test_successful_initialization(self) -> None:
        """Test successful security scanner initialization."""
        scanner = SecurityScanner()

        assert scanner.active_scans == {}
        assert scanner.alert_history == []
        assert scanner.vulnerability_db == {}

    def test_initialization_with_custom_config(self) -> None:
        """Test initialization with custom configuration."""
        config = {
            "scan_interval": 300,
            "alert_threshold": "HIGH",
            "enable_real_time_monitoring": True
        }

        scanner = SecurityScanner(config)

        assert scanner.scan_interval == 300
        assert scanner.alert_threshold == "HIGH"
        assert scanner.enable_real_time_monitoring is True


class TestSecurityScannerVulnerabilityScanning:
    """Test vulnerability scanning functionality."""

    def test_scan_single_vulnerability(self) -> None:
        """Test scanning for a single vulnerability."""
        scanner = SecurityScanner()

        # Mock vulnerability detection
        vulnerability = Vulnerability(
            id="CVE-2023-1234",
            severity="HIGH",
            description="Test vulnerability",
            affected_components=["database"],
            cvss_score=8.5,
            remediation="Update to latest version"
        )

        with patch.object(scanner, '_detect_vulnerabilities') as mock_detect:
            mock_detect.return_value = [vulnerability]

            result = scanner.scan_for_vulnerabilities("database")

            assert result.component == "database"
            assert len(result.vulnerabilities) == 1
            assert result.vulnerabilities[0].id == "CVE-2023-1234"

    def test_scan_multiple_components(self) -> None:
        """Test scanning multiple components for vulnerabilities."""
        scanner = SecurityScanner()

        vulnerabilities = [
            Vulnerability(
                id="CVE-2023-1234",
                severity="HIGH",
                description="Database vulnerability",
                affected_components=["database"],
                cvss_score=8.5,
                remediation="Update database"
            ),
            Vulnerability(
                id="CVE-2023-5678",
                severity="MEDIUM",
                description="API vulnerability",
                affected_components=["api"],
                cvss_score=6.2,
                remediation="Update API framework"
            )
        ]

        with patch.object(scanner, '_detect_vulnerabilities') as mock_detect:
            mock_detect.return_value = vulnerabilities

            result = scanner.scan_for_vulnerabilities(["database", "api"])

            assert len(result.vulnerabilities) == 2
            assert result.vulnerabilities[0].affected_components == ["database"]
            assert result.vulnerabilities[1].affected_components == ["api"]

    def test_scan_with_no_vulnerabilities_found(self) -> None:
        """Test scanning when no vulnerabilities are found."""
        scanner = SecurityScanner()

        with patch.object(scanner, '_detect_vulnerabilities') as mock_detect:
            mock_detect.return_value = []

            result = scanner.scan_for_vulnerabilities("clean_component")

            assert result.component == "clean_component"
            assert len(result.vulnerabilities) == 0
            assert result.scan_timestamp is not None


class TestSecurityScannerThreatDetection:
    """Test threat detection functionality."""

    def test_detect_threats_real_time(self) -> None:
        """Test real-time threat detection."""
        scanner = SecurityScanner({"enable_real_time_monitoring": True})
        logs = ["INFO: User login successful", "ERROR: Failed login attempt"]

        threat = ThreatAlert(
            id="THREAT-001",
            type="SUSPICIOUS_LOGIN",
            severity="HIGH",
            description="Multiple failed login attempts",
            source_ip="192.168.1.100",
            timestamp="2023-12-01T10:30:00Z",
            affected_systems=["auth_service"]
        )

        with patch.object(scanner, '_monitor_threats') as mock_monitor:
            mock_monitor.return_value = [threat]

            alerts = scanner.detect_threats(logs)

            assert len(alerts) == 1
            assert alerts[0].type == "SUSPICIOUS_LOGIN"
            assert alerts[0].severity == "HIGH"

    def test_detect_multiple_threats(self) -> None:
        """Test detection of multiple simultaneous threats."""
        scanner = SecurityScanner()
        logs = ["ERROR: Failed login", "WARN: High API traffic"]

        threats = [
            ThreatAlert(
                id="THREAT-001",
                type="SUSPICIOUS_LOGIN",
                severity="HIGH",
                description="Multiple failed logins",
                source_ip="192.168.1.100",
                timestamp="2023-12-01T10:30:00Z",
                affected_systems=["auth"]
            ),
            ThreatAlert(
                id="THREAT-002",
                type="UNUSUAL_API_TRAFFIC",
                severity="MEDIUM",
                description="High API call volume",
                source_ip="10.0.0.50",
                timestamp="2023-12-01T10:31:00Z",
                affected_systems=["api_gateway"]
            )
        ]

        with patch.object(scanner, '_monitor_threats') as mock_monitor:
            mock_monitor.return_value = threats

            alerts = scanner.detect_threats(logs)

            assert len(alerts) == 2
            assert alerts[0].id == "THREAT-001"
            assert alerts[1].id == "THREAT-002"

    def test_threat_detection_disabled(self) -> None:
        """Test threat detection when disabled in configuration."""
        scanner = SecurityScanner({"enable_real_time_monitoring": False})
        logs = ["INFO: Normal activity"]

        with patch.object(scanner, '_monitor_threats') as mock_monitor:
            mock_monitor.return_value = []

            alerts = scanner.detect_threats(logs)

            assert len(alerts) == 0
            mock_monitor.assert_not_called()


class TestSecurityScannerSecurityValidation:
    """Test security validation functionality."""

    def test_validate_security_policy_compliant(self) -> None:
        """Test security policy validation for compliant system."""
        scanner = SecurityScanner()

        policy_result = {
            "firewall_active": True,
            "encryption_enabled": True,
            "access_control_configured": True,
            "audit_logging_enabled": True
        }

        with patch.object(scanner, '_check_security_policies') as mock_check:
            mock_check.return_value = policy_result

            result = scanner.validate_security_policies()

            assert result["overall_status"] == "COMPLIANT"
            assert all(result.values())

    def test_validate_security_policy_non_compliant(self) -> None:
        """Test security policy validation for non-compliant system."""
        scanner = SecurityScanner()

        # Create violations for non-compliant system
        violations = [
            SecurityViolation(
                id="VIOLATION-001",
                type="MISSING_ENCRYPTION",
                severity="CRITICAL",
                description="Encryption not enabled",
                affected_component="database",
                remediation="Enable encryption",
                timestamp="2023-12-01T10:35:00Z"
            ),
            SecurityViolation(
                id="VIOLATION-002",
                type="MISSING_AUDIT_LOGGING",
                severity="HIGH",
                description="Audit logging not enabled",
                affected_component="system",
                remediation="Enable audit logging",
                timestamp="2023-12-01T10:36:00Z"
            )
        ]

        with patch.object(scanner, '_validate_security_controls') as mock_validate:
            mock_validate.return_value = violations

            result = scanner.validate_security_policies()

            assert result["overall_status"] == "NON_COMPLIANT"
            assert result["violations"] == violations

    def test_validate_security_policy_with_violations(self) -> None:
        """Test security validation with specific violations."""
        scanner = SecurityScanner()

        violations = [
            SecurityViolation(
                id="VIOLATION-001",
                type="MISSING_ENCRYPTION",
                severity="CRITICAL",
                description="Database encryption not enabled",
                affected_component="database",
                remediation="Enable encryption at rest",
                timestamp="2023-12-01T10:35:00Z"
            )
        ]

        with patch.object(scanner, '_validate_security_controls') as mock_validate:
            mock_validate.return_value = violations

            result = scanner.validate_security_controls()

            assert len(result) == 1
            assert result[0].type == "MISSING_ENCRYPTION"
            assert result[0].severity == "CRITICAL"


class TestSecurityScannerAPIIntegration:
    """Test integration with real APIs for security scanning."""

    def test_scan_external_api_endpoints(self) -> None:
        """Test scanning external API endpoints for vulnerabilities."""
        scanner = SecurityScanner()

        api_endpoints = [
            "https://api.coinbase.com/v2/exchange-rates",
            "https://api.binance.us/api/v3/ticker/price",
            "https://api.oanda.com/v3/instruments/EUR_USD/candles"
        ]

        with patch.object(scanner, '_scan_api_endpoint') as mock_scan:
            mock_scan.return_value = ScanResult(
                component="api_endpoint",
                vulnerabilities=[],
                scan_timestamp="2023-12-01T10:40:00Z"
            )

            results = scanner.scan_api_endpoints(api_endpoints)

            assert len(results) == 3
            mock_scan.assert_called()

    def test_detect_api_rate_limiting(self) -> None:
        """Test detection of API rate limiting issues."""
        scanner = SecurityScanner()

        rate_limit_info = {
            "requests_per_minute": 1000,
            "burst_limit": 100,
            "current_usage": 950,
            "is_approaching_limit": True
        }

        with patch.object(scanner, '_check_rate_limits') as mock_check:
            mock_check.return_value = rate_limit_info

            result = scanner.check_api_rate_limits(["coinbase", "binance"])

            assert result["is_approaching_limit"] is True
            assert result["current_usage"] == 950


class TestSecurityScannerAlertManagement:
    """Test alert management and notification functionality."""

    def test_generate_security_alert(self) -> None:
        """Test generation of security alerts."""
        scanner = SecurityScanner()

        alert = ThreatAlert(
            id="ALERT-001",
            type="SECURITY_BREACH",
            severity="CRITICAL",
            description="Potential security breach detected",
            source_ip="203.0.113.1",
            timestamp="2023-12-01T10:45:00Z",
            affected_systems=["web_server"]
        )

        with patch.object(scanner, '_send_alert_notification') as mock_send:
            mock_send.return_value = True

            result = scanner.generate_alert(alert)

            assert result is True
            assert alert.id in [a.id for a in scanner.alert_history]

    def test_alert_escalation_high_severity(self) -> None:
        """Test automatic escalation for high-severity alerts."""
        scanner = SecurityScanner({"alert_threshold": "MEDIUM"})

        critical_alert = ThreatAlert(
            id="CRITICAL-001",
            type="DATA_BREACH",
            severity="CRITICAL",
            description="Confirmed data breach",
            source_ip="203.0.113.1",
            timestamp="2023-12-01T10:50:00Z",
            affected_systems=["database"]
        )

        with patch.object(scanner, '_escalate_alert') as mock_escalate:
            mock_escalate.return_value = True

            scanner.process_alert(critical_alert)

            mock_escalate.assert_called_once()

    def test_alert_filtering_low_severity(self) -> None:
        """Test filtering out low-severity alerts."""
        scanner = SecurityScanner({"alert_threshold": "HIGH"})

        low_alert = ThreatAlert(
            id="LOW-001",
            type="UNUSUAL_ACTIVITY",
            severity="LOW",
            description="Minor unusual activity",
            source_ip="192.168.1.50",
            timestamp="2023-12-01T10:55:00Z",
            affected_systems=["monitoring"]
        )

        with patch.object(scanner, '_escalate_alert') as mock_escalate:
            mock_escalate.return_value = False

            scanner.process_alert(low_alert)

            mock_escalate.assert_not_called()


class TestSecurityScannerRiskAssessment:
    """Test risk assessment functionality."""

    def test_assess_overall_security_risk(self) -> None:
        """Test overall security risk assessment."""
        scanner = SecurityScanner()

        risk_factors = {
            "vulnerability_score": 7.5,
            "threat_level": "HIGH",
            "compliance_status": "PARTIAL",
            "incident_history": 3
        }

        with patch.object(scanner, '_calculate_risk_score') as mock_calculate:
            mock_calculate.return_value = risk_factors

            risk_assessment = scanner.assess_security_risk()

            assert risk_assessment["vulnerability_score"] == 7.5
            assert risk_assessment["threat_level"] == "HIGH"
            assert "overall_risk" in risk_assessment

    def test_generate_risk_report(self) -> None:
        """Test generation of comprehensive risk report."""
        scanner = SecurityScanner()

        with patch.object(scanner, 'assess_security_risk') as mock_assess:
            mock_assess.return_value = {
                "overall_risk": "HIGH",
                "vulnerability_score": 8.2,
                "threat_level": "CRITICAL",
                "recommendations": ["Update security patches", "Review access controls"]
            }

            report = scanner.generate_risk_report()

            assert report["overall_risk"] == "HIGH"
            assert len(report["recommendations"]) == 2
            assert report["generated_at"] is not None


class TestSecurityScannerIntegration:
    """Integration tests for SecurityScanner."""

    def test_full_security_scan_cycle(self) -> None:
        """Test complete security scan cycle."""
        scanner = SecurityScanner()

        # Mock all required methods
        with patch.object(scanner, 'scan_for_vulnerabilities') as mock_scan, \
             patch.object(scanner, 'detect_threats') as mock_detect, \
             patch.object(scanner, 'validate_security_policies') as mock_validate:

            mock_scan.return_value = ScanResult(
                component="system",
                vulnerabilities=[],
                scan_timestamp="2023-12-01T11:00:00Z"
            )
            mock_detect.return_value = []
            mock_validate.return_value = {"overall_status": "COMPLIANT"}

            result = scanner.perform_full_security_scan()

            assert result["scan_completed"] is True
            assert result["vulnerabilities_found"] == 0
            assert result["threats_detected"] == 0

    def test_security_scan_with_findings(self) -> None:
        """Test security scan that discovers vulnerabilities and threats."""
        scanner = SecurityScanner({"_test_with_findings": True})

        vulnerability = Vulnerability(
            id="CVE-2023-9999",
            severity="CRITICAL",
            description="Critical system vulnerability",
            affected_components=["core"],
            cvss_score=9.8,
            remediation="Immediate patching required"
        )

        threat = ThreatAlert(
            id="THREAT-999",
            type="MALICIOUS_ACTIVITY",
            severity="CRITICAL",
            description="Malicious activity detected",
            source_ip="203.0.113.195",
            timestamp="2023-12-01T11:05:00Z",
            affected_systems=["network"]
        )

        with patch.object(scanner, 'scan_for_vulnerabilities') as mock_scan, \
             patch.object(scanner, 'detect_threats') as mock_detect, \
             patch.object(scanner, 'validate_security_policies') as mock_validate:

            mock_scan.return_value = ScanResult(
                component="system",
                vulnerabilities=[vulnerability],
                scan_timestamp="2023-12-01T11:05:00Z"
            )
            mock_detect.return_value = [threat]
            mock_validate.return_value = {"overall_status": "NON_COMPLIANT"}

            result = scanner.perform_full_security_scan()

            assert result["vulnerabilities_found"] == 1
            assert result["threats_detected"] == 1
            assert result["overall_status"] == "NON_COMPLIANT"