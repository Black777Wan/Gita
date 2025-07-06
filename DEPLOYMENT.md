# Gita Deployment Guide

This guide provides comprehensive instructions for deploying Gita in production environments.

## 🚀 Quick Start

### Prerequisites
1. **Datomic Pro** - Licensed version required for production
2. **Java Runtime** - Java 8 or later
3. **System Dependencies** - Platform-specific libraries

### Installation Steps

1. **Download the release package**
   ```bash
   # Download from releases page or build from source
   wget https://github.com/your-org/gita/releases/latest/gita-production.tar.gz
   tar -xzf gita-production.tar.gz
   cd gita-production
   ```

2. **Run the installation script**
   ```bash
   # Linux/macOS
   ./install.sh
   
   # Windows
   install.bat
   ```

3. **Configure the application**
   ```bash
   # Edit configuration file
   nano ~/.local/share/gita/gita-config.toml
   ```

4. **Start Datomic transactor**
   ```bash
   cd /path/to/datomic-pro
   bin/transactor config/samples/dev-transactor-template.properties
   ```

5. **Launch Gita**
   ```bash
   gita
   ```

## 🔧 Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GITA_DB_URI` | Datomic database URI | `datomic:dev://localhost:8998/gita` |
| `GITA_DB_HOST` | Transactor host | `localhost` |
| `GITA_DB_PORT` | Transactor port | `8998` |
| `DATOMIC_LIB_PATH` | Path to Datomic lib directory | Auto-detected |
| `GITA_LOG_LEVEL` | Log level (trace, debug, info, warn, error) | `info` |
| `GITA_DATA_DIR` | Data directory | `~/.local/share/gita` |

### Configuration File

Create `gita-config.toml` in your data directory:

```toml
[datomic]
db_uri = "datomic:dev://localhost:8998/gita"
transactor_host = "localhost"
transactor_port = 8998
database_name = "gita"
datomic_lib_path = "/path/to/datomic-pro/lib"
connection_timeout_ms = 30000
retry_attempts = 3
jvm_opts = ["-Xmx4g", "-Xms1g", "-XX:+UseG1GC"]

[audio]
recordings_dir = "recordings"
max_recording_duration_minutes = 120
sample_rate = 44100
channels = 2

log_level = "info"
data_dir = "/path/to/data"
```

## 🏗️ Building from Source

### Prerequisites
- Rust 1.70+ (`rustup.rs`)
- Node.js 18+ (`nodejs.org`)
- Datomic Pro installation

### Build Steps

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-org/gita.git
   cd gita
   ```

2. **Run the build script**
   ```bash
   # Linux/macOS
   chmod +x build-production.sh
   ./build-production.sh
   
   # Windows
   build-production.bat
   ```

3. **The built application will be in `dist/`**

### Manual Build

```bash
# Install frontend dependencies
cd frontend
npm install
npm run build
cd ..

# Build Rust application
cd src-tauri
cargo build --release
cd ..

# Copy files to dist
mkdir -p dist
cp src-tauri/target/release/gita dist/
cp README.md dist/
```

## 🐳 Docker Deployment

### Dockerfile

```dockerfile
FROM ubuntu:22.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    openjdk-17-jre \
    curl \
    wget \
    build-essential \
    libwebkit2gtk-4.0-dev \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libasound2-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -s /bin/bash gita

# Set up Datomic
COPY datomic-pro.tar.gz /tmp/
RUN cd /opt && tar -xzf /tmp/datomic-pro.tar.gz && \
    chown -R gita:gita /opt/datomic-pro*

# Copy application
COPY dist/gita /usr/local/bin/
COPY dist/gita-config.toml /etc/gita/
RUN chmod +x /usr/local/bin/gita

# Switch to app user
USER gita
WORKDIR /home/gita

# Set environment variables
ENV GITA_DATA_DIR=/home/gita/.local/share/gita
ENV DATOMIC_LIB_PATH=/opt/datomic-pro/lib

# Expose ports
EXPOSE 8080

# Start script
COPY start.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/start.sh

CMD ["/usr/local/bin/start.sh"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  datomic:
    image: ubuntu:22.04
    volumes:
      - ./datomic-pro:/opt/datomic-pro
      - datomic-data:/var/lib/datomic
    ports:
      - "8998:8998"
    command: |
      bash -c "
        apt-get update && apt-get install -y openjdk-17-jre &&
        cd /opt/datomic-pro &&
        bin/transactor config/samples/dev-transactor-template.properties
      "
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8998/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  gita:
    build: .
    ports:
      - "8080:8080"
    depends_on:
      - datomic
    environment:
      - GITA_DB_URI=datomic:dev://datomic:8998/gita
      - GITA_DB_HOST=datomic
      - GITA_LOG_LEVEL=info
    volumes:
      - gita-data:/home/gita/.local/share/gita
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  datomic-data:
  gita-data:
```

## 🎯 Production Deployment

### System Requirements

**Minimum:**
- 2 CPU cores
- 4 GB RAM
- 10 GB storage
- Java 8+

**Recommended:**
- 4 CPU cores
- 8 GB RAM
- 50 GB storage
- Java 17+

### Performance Tuning

#### JVM Options
```toml
jvm_opts = [
  "-Xmx4g",
  "-Xms2g",
  "-XX:+UseG1GC",
  "-XX:MaxGCPauseMillis=200",
  "-XX:+UseStringDeduplication",
  "-XX:+OptimizeStringConcat"
]
```

#### Datomic Configuration
```properties
# transactor.properties
protocol=dev
host=localhost
port=8998
memory-index-threshold=32m
memory-index-max=512m
object-cache-max=1g
```

### Monitoring

#### Health Checks
```bash
# Check application health
curl http://localhost:8080/health

# Check Datomic health
curl http://localhost:8998/health
```

#### Log Monitoring
```bash
# View application logs
tail -f ~/.local/share/gita/logs/gita.log

# View Datomic logs
tail -f /path/to/datomic-pro/log/transactor.log
```

### Backup Strategy

#### Database Backup
```bash
# Backup Datomic database
cd /path/to/datomic-pro
bin/datomic backup-db datomic:dev://localhost:8998/gita file:backup/gita-$(date +%Y%m%d)
```

#### Audio Files Backup
```bash
# Backup audio recordings
rsync -av ~/.local/share/gita/recordings/ /backup/gita-audio/
```

### Security Considerations

1. **Database Security**
   - Use strong passwords for Datomic
   - Enable SSL/TLS for database connections
   - Restrict network access to database port

2. **Application Security**
   - Run application as non-root user
   - Use firewall to restrict access
   - Enable audit logging

3. **File System Security**
   - Secure audio file storage
   - Regular security updates
   - Proper file permissions

### Scaling

#### Horizontal Scaling
- Use Datomic clustering for high availability
- Load balance multiple application instances
- Shared file system for audio recordings

#### Vertical Scaling
- Increase JVM heap size
- Add more CPU cores
- Increase storage capacity

## 🔍 Troubleshooting

### Common Issues

#### "JVM initialization failed"
```bash
# Check Java installation
java -version

# Check Datomic classpath
ls -la $DATOMIC_LIB_PATH

# Verify JVM options
cat gita-config.toml | grep jvm_opts
```

#### "Connection refused"
```bash
# Check if transactor is running
netstat -an | grep 8998

# Check firewall
sudo ufw status

# Test connection
telnet localhost 8998
```

#### "Schema transaction failed"
```bash
# Check transactor logs
tail -f /path/to/datomic-pro/log/transactor.log

# Verify database permissions
# Check disk space
df -h
```

### Debug Mode

Enable debug logging:
```toml
log_level = "debug"
```

Or use environment variable:
```bash
GITA_LOG_LEVEL=debug gita
```

### Performance Issues

1. **Check resource usage**
   ```bash
   top
   free -h
   iostat
   ```

2. **Monitor JVM metrics**
   ```bash
   # Add JVM monitoring flags
   jvm_opts = [
     "-XX:+PrintGCDetails",
     "-XX:+PrintGCTimeStamps",
     "-Xloggc:gc.log"
   ]
   ```

3. **Profile application**
   ```bash
   # Use profiling tools
   perf top -p $(pgrep gita)
   ```

## 📞 Support

For additional support:
- Check the [GitHub Issues](https://github.com/your-org/gita/issues)
- Review the [FAQ](FAQ.md)
- Contact support team

## 🔄 Updates

To update Gita:
1. Download the latest release
2. Stop the application
3. Backup configuration and data
4. Install new version
5. Restart application

Automated updates coming soon!
