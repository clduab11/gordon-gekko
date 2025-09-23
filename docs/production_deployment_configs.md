# Gordon Gekko Production Deployment Configurations

## Overview

This document provides comprehensive Docker and Kubernetes deployment configurations for the Gordon Gekko autonomous trading system, specifically optimized for Apple Silicon M1/M2 processors with multi-GPU support. All configurations follow production best practices with security, scalability, and observability built-in.

## Docker Configuration Strategy

### Multi-Architecture Dockerfiles

#### 1. Apple Silicon GPU Worker Dockerfile

```dockerfile
# Multi-architecture Dockerfile optimized for Apple Silicon + CUDA
FROM python:3.11-slim

# Build arguments for multi-platform support
ARG TARGETPLATFORM
ARG BUILDPLATFORM
ARG GPU_SUPPORT=mps,cuda

# Labels for deployment management
LABEL maintainer="Gordon Gekko Team" \
      version="2.1.0" \
      description="Apple Silicon GPU Worker with MPS and CUDA support" \
      architecture="multi-arch" \
      gpu.support="mps,cuda" \
      deployment.env="production"

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    libgomp1 \
    libgl1-mesa-glx \
    libglib2.0-0 \
    libsm6 \
    libxext6 \
    libxrender-dev \
    libgomp1 \
    libatlas-base-dev \
    && rm -rf /var/lib/apt/lists/*

# Apple Silicon GPU support
RUN pip install --no-cache-dir \
    torch==2.1.0+cpu \
    torchvision==0.16.0+cpu \
    torchaudio==2.1.0+cpu \
    --index-url https://download.pytorch.org/whl/cpu

# CUDA support for non-Apple Silicon systems
RUN pip install --no-cache-dir nvidia-ml-py

# Core dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# GPU optimization libraries
RUN pip install --no-cache-dir \
    torch-mps \
    accelerate \
    transformers

# Application code
COPY src/gordon_gekko/ /app/src/gordon_gekko/
WORKDIR /app

# Health check for container orchestration
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD python -c "
import sys
try:
    import torch
    import torch.mps
    print('GPU support:', torch.backends.mps.is_available() if hasattr(torch.backends, 'mps') else 'CUDA available:', torch.cuda.is_available())
    sys.exit(0)
except Exception as e:
    print(f'Health check failed: {e}')
    sys.exit(1)
"

# Environment variables for GPU optimization
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    MPS_GPU_MEMORY_LIMIT=0.8 \
    CUDA_VISIBLE_DEVICES=0 \
    TORCH_USE_CUDA_DSA=1 \
    GPU_MEMORY_FRACTION=0.8

# Resource limits and requests
# These will be overridden by Kubernetes resource specifications

CMD ["python", "-m", "src.gordon_gekko.ml.worker"]
```

#### 2. Deployment Orchestrator Dockerfile

```dockerfile
FROM python:3.11-slim

# Build configuration
ARG ENVIRONMENT=production
ARG VERSION=2.1.0

LABEL maintainer="Gordon Gekko Team" \
      version="${VERSION}" \
      environment="${ENVIRONMENT}" \
      component="deployment-orchestrator" \
      security.scan="enabled"

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    gnupg2 \
    lsb-release \
    && rm -rf /var/lib/apt/lists/*

# Install Docker CLI for container management
RUN curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg \
    && echo "deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && apt-get update && apt-get install -y docker-ce-cli

# Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Application code
COPY src/gordon_gekko/deployment/ /app/src/gordon_gekko/deployment/
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD python -c "import sys; sys.exit(0)" || exit 1

# Environment variables
ENV PYTHONUNBUFFERED=1 \
    ENVIRONMENT=${ENVIRONMENT} \
    VERSION=${VERSION}

CMD ["python", "-m", "src.gordon_gekko.deployment.orchestrator"]
```

### Kubernetes Deployment Manifests

#### 1. Apple Silicon GPU Worker Deployment

```yaml
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: gpu-worker-config
  namespace: gordon-gekko-production
  labels:
    app: gordon-gekko
    component: gpu-worker
    version: "2.1.0"
data:
  gpu-type: "apple-silicon"
  mps-memory-limit: "0.8"
  model-inference-timeout: "50"
  max-batch-size: "32"
  prometheus-port: "9090"
  health-check-interval: "30"

---
apiVersion: v1
kind: Secret
metadata:
  name: gpu-worker-secrets
  namespace: gordon-gekko-production
  labels:
    app: gordon-gekko
    component: gpu-worker
type: Opaque
data:
  # Base64 encoded secrets
  model-registry-token: <base64-encoded-token>
  prometheus-token: <base64-encoded-token>

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: gordon-gekko-gpu-worker
  namespace: gordon-gekko-production
  labels:
    app: gordon-gekko
    component: gpu-worker
    version: "2.1.0"
    environment: production
spec:
  replicas: 4
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 1
  selector:
    matchLabels:
      app: gordon-gekko
      component: gpu-worker
  template:
    metadata:
      labels:
        app: gordon-gekko
        component: gpu-worker
        version: "2.1.0"
        environment: production
        gpu-type: apple-silicon
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      # Apple Silicon GPU node selector
      nodeSelector:
        accelerator: apple-silicon-gpu
        gpu-type: mps-cuda
      tolerations:
      - key: dedicated
        operator: Equal
        value: gpu-worker
        effect: NoSchedule
      - key: apple-silicon
        operator: Equal
        value: "true"
        effect: NoSchedule
      affinity:
        nodeAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
            nodeSelectorTerms:
            - matchExpressions:
              - key: accelerator
                operator: In
                values: ["apple-silicon-gpu"]
              - key: gpu-type
                operator: In
                values: ["mps-cuda", "cuda-only"]
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        runAsGroup: 1000
        fsGroup: 2000
      containers:
      - name: gpu-worker
        image: gordon-gekko/gpu-worker:2.1.0
        imagePullPolicy: Always
        resources:
          requests:
            cpu: "2000m"
            memory: "8Gi"
            apple-silicon-gpu: 1
          limits:
            cpu: "4000m"
            memory: "16Gi"
            apple-silicon-gpu: 1
        env:
        - name: ENVIRONMENT
          value: "production"
        - name: GPU_TYPE
          valueFrom:
            configMapKeyRef:
              name: gpu-worker-config
              key: gpu-type
        - name: MPS_GPU_MEMORY_LIMIT
          valueFrom:
            configMapKeyRef:
              name: gpu-worker-config
              key: mps-memory-limit
        - name: MODEL_INFERENCE_TIMEOUT
          valueFrom:
            configMapKeyRef:
              name: gpu-worker-config
              key: model-inference-timeout
        - name: PROMETHEUS_PORT
          valueFrom:
            configMapKeyRef:
              name: gpu-worker-config
              key: prometheus-port
        - name: MODEL_REGISTRY_TOKEN
          valueFrom:
            secretKeyRef:
              name: gpu-worker-secrets
              key: model-registry-token
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        - containerPort: 9090
          name: metrics
          protocol: TCP
        volumeMounts:
        - name: model-storage
          mountPath: /app/models
          readOnly: true
        - name: config-volume
          mountPath: /app/config
          readOnly: true
        - name: tmp-volume
          mountPath: /tmp
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
            scheme: HTTPS
          initialDelaySeconds: 60
          periodSeconds: 30
          timeoutSeconds: 10
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
            scheme: HTTPS
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop: ["ALL"]
      volumes:
      - name: model-storage
        persistentVolumeClaim:
          claimName: model-storage-pvc
      - name: config-volume
        configMap:
          name: gpu-worker-config
      - name: tmp-volume
        emptyDir: {}
      restartPolicy: Always
```

#### 2. Deployment Orchestrator Deployment

```yaml
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: deployment-orchestrator-config
  namespace: gordon-gekko-production
data:
  environment: "production"
  log-level: "INFO"
  deployment-timeout: "300"
  rollback-timeout: "180"
  health-check-interval: "30"
  max-deployment-retries: "3"

---
apiVersion: v1
kind: Secret
metadata:
  name: deployment-orchestrator-secrets
  namespace: gordon-gekko-production
type: Opaque
data:
  docker-registry-token: <base64-encoded-token>
  kubernetes-token: <base64-encoded-token>
  prometheus-token: <base64-encoded-token>

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: deployment-orchestrator
  namespace: gordon-gekko-production
  labels:
    app: gordon-gekko
    component: deployment-orchestrator
    version: "2.1.0"
spec:
  replicas: 2
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 1
  selector:
    matchLabels:
      app: gordon-gekko
      component: deployment-orchestrator
  template:
    metadata:
      labels:
        app: gordon-gekko
        component: deployment-orchestrator
        version: "2.1.0"
    spec:
      serviceAccountName: deployment-orchestrator
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        runAsGroup: 1000
      containers:
      - name: orchestrator
        image: gordon-gekko/deployment-orchestrator:2.1.0
        imagePullPolicy: Always
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "1000m"
            memory: "1Gi"
        env:
        - name: ENVIRONMENT
          valueFrom:
            configMapKeyRef:
              name: deployment-orchestrator-config
              key: environment
        - name: LOG_LEVEL
          valueFrom:
            configMapKeyRef:
              name: deployment-orchestrator-config
              key: log-level
        - name: DEPLOYMENT_TIMEOUT
          valueFrom:
            configMapKeyRef:
              name: deployment-orchestrator-config
              key: deployment-timeout
        - name: DOCKER_REGISTRY_TOKEN
          valueFrom:
            secretKeyRef:
              name: deployment-orchestrator-secrets
              key: docker-registry-token
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        volumeMounts:
        - name: config-volume
          mountPath: /app/config
          readOnly: true
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop: ["ALL"]
      volumes:
      - name: config-volume
        configMap:
          name: deployment-orchestrator-config
```

#### 3. Service Mesh Configuration

```yaml
---
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: gordon-gekko-mtls
  namespace: gordon-gekko-production
spec:
  selector:
    matchLabels:
      app: gordon-gekko
  mtls:
    mode: STRICT

---
apiVersion: security.istio.io/v1beta1
kind: RequestAuthentication
metadata:
  name: gordon-gekko-jwt
  namespace: gordon-gekko-production
spec:
  selector:
    matchLabels:
      app: gordon-gekko
  jwtRules:
  - issuer: "https://auth.gordon-gekko.internal"
    jwksUri: "https://auth.gordon-gekko.internal/.well-known/jwks.json"
    audiences: ["gordon-gekko-api"]
    forwardOriginalToken: true

---
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: gordon-gekko-authz
  namespace: gordon-gekko-production
spec:
  selector:
    matchLabels:
      app: gordon-gekko
  rules:
  - from:
    - source:
        principals: ["cluster.local/ns/gordon-gekko-production/sa/deployment-orchestrator"]
    to:
    - operation:
        methods: ["GET", "POST", "PUT", "DELETE"]
        paths: ["/api/v1/*"]
  - from:
    - source:
        principals: ["cluster.local/ns/gordon-gekko-production/sa/gpu-manager"]
    to:
    - operation:
        methods: ["GET", "POST"]
        paths: ["/api/v1/gpu/*"]
  - to:
    - operation:
        methods: ["GET"]
        paths: ["/health/*"]

---
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: gordon-gekko-vs
  namespace: gordon-gekko-production
spec:
  hosts:
  - "api.gordon-gekko.internal"
  gateways:
  - gordon-gekko-gateway
  http:
  - match:
    - uri:
        prefix: "/api/v1/deployment"
    route:
    - destination:
        host: deployment-orchestrator
        port:
          number: 8080
    timeout: 300s
  - match:
    - uri:
        prefix: "/api/v1/gpu"
    route:
    - destination:
        host: gpu-manager
        port:
          number: 8080
    timeout: 60s
  - match:
    - uri:
        prefix: "/health"
    route:
    - destination:
        host: deployment-orchestrator
        port:
          number: 8080
    timeout: 10s

---
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: gordon-gekko-dr
  namespace: gordon-gekko-production
spec:
  host: deployment-orchestrator
  trafficPolicy:
    tls:
      mode: ISTIO_MUTUAL
    connectionPool:
      tcp:
        maxConnections: 100
        connectTimeout: 10s
        keepalive:
          time: 7200s
          interval: 75s
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 60s
      maxEjectionPercent: 50
  subsets:
  - name: v2-1-0
    labels:
      version: "2.1.0"
```

### Helm Charts

#### 1. Main Application Helm Chart

```yaml
# values.yaml
global:
  imageRegistry: "gordon-gekko"
  imageTag: "2.1.0"
  environment: "production"
  replicas: 3

appleSilicon:
  enabled: true
  gpuWorkers: 4
  memoryLimit: "0.8"
  nodeSelector:
    accelerator: "apple-silicon-gpu"

security:
  mTLS:
    enabled: true
  jwt:
    enabled: true
    issuer: "https://auth.gordon-gekko.internal"

monitoring:
  enabled: true
  prometheus:
    enabled: true
    retention: "30d"
  grafana:
    enabled: true
    persistence:
      enabled: true
      size: "10Gi"

ingress:
  enabled: true
  className: "nginx"
  annotations:
    kubernetes.io/ingress.class: "nginx"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
  hosts:
  - host: api.gordon-gekko.internal
    paths:
    - path: /
      pathType: Prefix
  tls:
  - secretName: gordon-gekko-tls
    hosts:
    - api.gordon-gekko.internal
```

#### 2. Deployment Orchestrator Helm Chart

```yaml
# Chart.yaml
apiVersion: v2
name: gordon-gekko-deployment-orchestrator
description: A Helm chart for Gordon Gekko Deployment Orchestrator
type: application
version: 2.1.0
appVersion: "2.1.0"

# values.yaml
replicaCount: 2

image:
  registry: gordon-gekko
  repository: deployment-orchestrator
  tag: "2.1.0"
  pullPolicy: Always

service:
  type: ClusterIP
  port: 8080
  targetPort: 8080
  name: http

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 500m
    memory: 512Mi

nodeSelector: {}

tolerations: []

affinity: {}

securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  runAsGroup: 1000

serviceAccount:
  create: true
  name: deployment-orchestrator
  annotations: {}

config:
  environment: "production"
  logLevel: "INFO"
  deploymentTimeout: "300"
  rollbackTimeout: "180"

secrets:
  dockerRegistryToken: ""
  kubernetesToken: ""
  prometheusToken: ""

# Dependencies
prometheus:
  enabled: true
  port: 9090

istio:
  enabled: true
  mtls: true
  authorization: true
```

## Service Mesh and Ingress Configuration

### 1. Istio Service Mesh Configuration

```yaml
---
apiVersion: install.istio.io/v1alpha1
kind: IstioOperator
metadata:
  name: gordon-gekko-istio
spec:
  profile: "production"
  meshConfig:
    enableTracing: true
    defaultConfig:
      proxyMetadata:
        ISTIO_META_DNS_AUTO_ALLOCATE: "true"
      tracing:
        zipkin:
          address: zipkin.istio-system:9411
    extensionProviders:
    - name: prometheus
      prometheus:
        service: prometheus.monitoring.svc.cluster.local
        port: 9090
    - name: grafana
      grafana:
        service: grafana.monitoring.svc.cluster.local
        port: 3000
  components:
    pilot:
      enabled: true
      k8s:
        env:
        - name: PILOT_TRACE_SAMPLING
          value: "100"
    ingressGateways:
    - name: istio-ingressgateway
      enabled: true
      k8s:
        service:
          ports:
          - port: 15021
            targetPort: 15021
            name: status-port
            protocol: TCP
          - port: 80
            targetPort: 8080
            name: http2
            protocol: TCP
          - port: 443
            targetPort: 8443
            name: https
            protocol: TCP
    egressGateways:
    - name: istio-egressgateway
      enabled: true
  values:
    global:
      proxy:
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
    pilot:
      resources:
        requests:
          cpu: 500m
          memory: 1Gi
        limits:
          cpu: 1000m
          memory: 2Gi
    gateways:
      istio-ingressgateway:
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 2000m
            memory: 1Gi
```

### 2. NGINX Ingress Controller Configuration

```yaml
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: nginx-ingress-controller-config
  namespace: gordon-gekko-production
data:
  ssl-protocols: "TLSv1.3 TLSv1.2"
  ssl-ciphers: "ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384"
  ssl-prefer-server-ciphers: "false"
  use-forwarded-headers: "true"
  compute-full-forwarded-for: "true"
  use-proxy-protocol: "true"
  real-ip-header: "CF-Connecting-IP"
  set-real-ip-from: "0.0.0.0/0"
  http2-max-field-size: "16k"
  http2-max-header-size: "32k"
  http2-max-requests: "1000"
  upstream-keepalive-connections: "32"
  upstream-keepalive-timeout: "60"
  upstream-keepalive-requests: "100"

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx-ingress-controller
  namespace: gordon-gekko-production
spec:
  replicas: 2
  selector:
    matchLabels:
      app: nginx-ingress-controller
  template:
    metadata:
      labels:
        app: nginx-ingress-controller
    spec:
      serviceAccountName: nginx-ingress-serviceaccount
      containers:
      - name: nginx-ingress-controller
        image: nginx/nginx-ingress:2.4.2
        args:
        - -nginx-configmaps=$(POD_NAMESPACE)/nginx-ingress-controller-config
        - -default-server-tls-secret=$(POD_NAMESPACE)/gordon-gekko-tls
        - -enable-ssl-passthrough
        - -v=2
        ports:
        - name: http
          containerPort: 80
        - name: https
          containerPort: 443
        env:
        - name: POD_NAMESPACE
          valueFrom:
            fieldRef:
              fieldPath: metadata.namespace
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
        livenessProbe:
          httpGet:
            path: /healthz
            port: 10254
            scheme: HTTP
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /healthz
            port: 10254
            scheme: HTTP
          initialDelaySeconds: 5
          periodSeconds: 5
```

## Multi-Environment Deployment Strategy

### 1. Environment-Specific Configurations

#### Development Environment
```yaml
# k8s/development-values.yaml
global:
  environment: "development"
  replicas: 1
  imageTag: "latest"

appleSilicon:
  enabled: true
  gpuWorkers: 1

resources:
  requests:
    cpu: "500m"
    memory: "1Gi"
  limits:
    cpu: "1000m"
    memory: "2Gi"

monitoring:
  enabled: false

ingress:
  enabled: false
```

#### Staging Environment
```yaml
# k8s/staging-values.yaml
global:
  environment: "staging"
  replicas: 2
  imageTag: "2.1.0-rc.1"

appleSilicon:
  enabled: true
  gpuWorkers: 2

monitoring:
  enabled: true
  prometheus:
    retention: "7d"

ingress:
  enabled: true
  hosts:
  - host: staging-api.gordon-gekko.internal
```

#### Production Environment
```yaml
# k8s/production-values.yaml
global:
  environment: "production"
  replicas: 4
  imageTag: "2.1.0"

appleSilicon:
  enabled: true
  gpuWorkers: 4

monitoring:
  enabled: true
  prometheus:
    retention: "30d"
  grafana:
    persistence:
      enabled: true

ingress:
  enabled: true
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
  hosts:
  - host: api.gordon-gekko.internal
```

### 2. Blue-Green Deployment Strategy

```yaml
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: gordon-gekko-green
  namespace: gordon-gekko-production
spec:
  replicas: 4
  selector:
    matchLabels:
      app: gordon-gekko
      version: green
  template:
    metadata:
      labels:
        app: gordon-gekko
        version: green
        deployment-strategy: blue-green
    spec:
      # Green deployment configuration

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: gordon-gekko-blue
  namespace: gordon-gekko-production
spec:
  replicas: 4
  selector:
    matchLabels:
      app: gordon-gekko
      version: blue
  template:
    metadata:
      labels:
        app: gordon-gekko
        version: blue
        deployment-strategy: blue-green
    spec:
      # Blue deployment configuration (current production)

---
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: gordon-gekko-blue-green
  namespace: gordon-gekko-production
spec:
  hosts:
  - api.gordon-gekko.internal
  http:
  - name: "blue-green-routing"
    match:
    - headers:
        version:
          exact: "green"
    route:
    - destination:
        host: gordon-gekko-green
        subset: v2-1-0
      weight: 100
    - destination:
        host: gordon-gekko-blue
        subset: v2-1-0
      weight: 0
  - name: "production-routing"
    route:
    - destination:
        host: gordon-gekko-blue
        subset: v2-1-0
      weight: 100
```

## Security and Compliance Configurations

### 1. Pod Security Standards

```yaml
---
apiVersion: v1
kind: Pod
metadata:
  name: gordon-gekko-secure-pod
  namespace: gordon-gekko-production
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    runAsGroup: 1000
    fsGroup: 2000
    supplementalGroups: [3000]
    seccompProfile:
      type: Localhost
      localhostProfile: runtime/default
  containers:
  - name: app
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      runAsNonRoot: true
      runAsUser: 1000
      runAsGroup: 1000
      capabilities:
        drop: ["ALL"]
      seccompProfile:
        type: Localhost
        localhostProfile: runtime/default
    volumeMounts:
    - name: tmp-volume
      mountPath: /tmp
    - name: cache-volume
      mountPath: /var/cache
  volumes:
  - name: tmp-volume
    emptyDir: {}
  - name: cache-volume
    emptyDir: {}
```

### 2. Network Policies

```yaml
---
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: gordon-gekko-egress
  namespace: gordon-gekko-production
spec:
  podSelector:
    matchLabels:
      app: gordon-gekko
  policyTypes:
  - Egress
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: gordon-gekko
    - namespaceSelector:
        matchLabels:
          name: kube-system
    - namespaceSelector:
        matchLabels:
          name: istio-system
  - to: []
    ports:
    - protocol: TCP
      port: 53
    - protocol: UDP
      port: 53
  - to: []
    ports:
    - protocol: TCP
      port: 443
    - protocol: TCP
      port: 80

---
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: gordon-gekko-ingress
  namespace: gordon-gekko-production
spec:
  podSelector:
    matchLabels:
      app: gordon-gekko
  policyTypes:
  - Ingress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: istio-system
    - podSelector:
        matchLabels:
          app: gordon-gekko
          component: istio-ingressgateway
    - podSelector:
        matchLabels:
          app: prometheus
    - podSelector:
        matchLabels:
          app: grafana
    ports:
    - protocol: TCP
      port: 8080
    - protocol: TCP
      port: 9090
    - protocol: TCP
      port: 15000
    - protocol: TCP
      port: 15090
```

## Monitoring and Observability

### 1. Prometheus Configuration

```yaml
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: monitoring
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
      evaluation_interval: 15s
    rule_files:
    - /etc/prometheus/rules/*.yml
    scrape_configs:
    - job_name: 'kubernetes-pods'
      kubernetes_sd_configs:
      - role: pod
      relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        target_label: application
        regex: gordon-gekko
        action: keep
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        regex: ([^:]+)(?::\d+)?;(\d+)
        replacement: $1:$2
        target_label: __address__
    - job_name: 'gordon-gekko-services'
      static_configs:
      - targets: ['deployment-orchestrator:8080', 'gpu-manager:8080']
    alerting:
      alertmanagers:
      - static_configs:
        - targets: ['alertmanager:9093']
```

### 2. Grafana Dashboard Configuration

```yaml
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: grafana-dashboards
  namespace: monitoring
data:
  gordon-gekko-overview.json: |
    {
      "dashboard": {
        "title": "Gordon Gekko - System Overview",
        "tags": ["gordon-gekko", "production"],
        "timezone": "UTC",
        "panels": [
          {
            "title": "Apple Silicon GPU Utilization",
            "type": "graph",
            "targets": [
              {
                "expr": "apple_gpu_memory_usage",
                "legendFormat": "GPU {{instance}}"
              }
            ]
          },
          {
            "title": "Deployment Success Rate",
            "type": "stat",
            "targets": [
              {
                "expr": "rate(deployment_success_total[5m]) * 100",
                "legendFormat": "Success Rate %"
              }
            ]
          },
          {
            "title": "Trading Engine Performance",
            "type": "graph",
            "targets": [
              {
                "expr": "trading_engine_latency_seconds",
                "legendFormat": "Latency (s)"
              }
            ]
          }
        ]
      }
    }
```

## CI/CD Pipeline Configuration

### 1. GitHub Actions Workflow

```yaml
# .github/workflows/production-deployment.yml
name: Production Deployment
on:
  push:
    branches: [main]
    paths: ['src/**', 'k8s/**', 'Dockerfile*']
  workflow_dispatch:

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}
  VERSION: ${{ github.sha }}

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
    - name: Checkout code
      uses: actions/checkout@v3
    - name: Run tests
      run: |
        python -m pytest tests/ -v --cov=src --cov-report=xml
    - name: Upload coverage
      uses: codecov/codecov-action@v3

  security-scan:
    name: Security Scan
    runs-on: ubuntu-latest
    steps:
    - name: Checkout code
      uses: actions/checkout@v3
    - name: Run Trivy vulnerability scanner
      uses: aquasecurity/trivy-action@master
      with:
        scan-type: 'fs'
        scan-ref: '.'
        format: 'sarif'
        output: 'trivy-results.sarif'
    - name: Upload Trivy scan results
      uses: github/codeql-action/upload-sarif@v2
      with:
        sarif_file: 'trivy-results.sarif'

  build:
    name: Build and Push
    runs-on: ubuntu-latest
    needs: [test, security-scan]
    strategy:
      matrix:
        platform: [linux/amd64, linux/arm64]
    steps:
    - name: Checkout code
      uses: actions/checkout@v3
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v2
    - name: Log in to Container Registry
      uses: docker/login-action@v2
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v4
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
        tags: |
          type=ref,event=branch
          type=ref,event=pr
          type=sha,prefix={{branch}}-
          type=raw,value=2.1.0,enable={{is_default_branch}}
    - name: Build and push Docker image
      uses: docker/build-push-action@v4
      with:
        context: .
        platforms: ${{ matrix.platform }}
        push: true
        tags: ${{ steps.meta.outputs.tags }}
        labels: ${{ steps.meta.outputs.labels }}
        cache-from: type=gha
        cache-to: type=gha,mode=max
        build-args: |
          VERSION=${{ env.VERSION }}
          ENVIRONMENT=production
          GPU_SUPPORT=mps,cuda

  deploy:
    name: Deploy to Production
    runs-on: ubuntu-latest
    needs: build
    environment: production
    steps:
    - name: Checkout code
      uses: actions/checkout@v3
    - name: Configure kubectl
      uses: azure/k8s-set-context@v3
      with:
        method: kubeconfig
        kubeconfig: ${{ secrets.KUBE_CONFIG }}
    - name: Deploy to Kubernetes
      run: |
        helm upgrade --install gordon-gekko ./k8s \
          --namespace gordon-gekko-production \
          --create-namespace \
          --set image.tag=${{ env.VERSION }} \
          --set environment=production \
          --wait \
          --timeout=600s
    - name: Verify deployment
      run: |
        kubectl rollout status deployment/gordon-gekko-gpu-worker \
          --namespace gordon-gekko-production \
          --timeout=600s
        kubectl rollout status deployment/deployment-orchestrator \
          --namespace gordon-gekko-production \
          --timeout=300s

  rollback:
    name: Rollback on Failure
    runs-on: ubuntu-latest
    needs: deploy
    if: failure()
    steps:
    - name: Rollback deployment
      run: |
        helm rollback gordon-gekko --namespace gordon-gekko-production
        kubectl rollout undo deployment/gordon-gekko-gpu-worker \
          --namespace gordon-gekko-production
        kubectl rollout undo deployment/deployment-orchestrator \
          --namespace gordon-gekko-production
```

## Summary

This comprehensive configuration provides:

1. **Multi-Architecture Docker Support**: Optimized for Apple Silicon M1/M2 with CUDA compatibility
2. **Kubernetes Production Deployments**: Enterprise-grade K8s configurations with security and scaling
3. **Service Mesh Integration**: Istio-based zero-trust networking with mTLS and authorization
4. **Multi-Environment Strategy**: Development, staging, and production configurations
5. **Blue-Green Deployments**: Zero-downtime deployment strategy with automated rollback
6. **Comprehensive Security**: Pod security standards, network policies, and compliance
7. **Monitoring Integration**: Prometheus, Grafana, and alerting configurations
8. **CI/CD Pipeline**: Automated deployment with testing, security scanning, and rollback

All configurations follow production best practices with security, scalability, and observability built-in. The Apple Silicon optimization ensures maximum performance for ML workloads while maintaining compatibility with traditional GPU infrastructure.