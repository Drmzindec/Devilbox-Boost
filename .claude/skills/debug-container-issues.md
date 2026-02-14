# Debug Container Issues

Troubleshoot common Devilbox container problems.

## Quick Health Check

```bash
# Check all services
docker-compose ps

# Check container health
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

# Check logs for errors
docker-compose logs --tail=50 php
docker-compose logs --tail=50 httpd
docker-compose logs --tail=50 mysql
```

## PHP Container Issues

### Container Won't Start

```bash
# Check logs
docker-compose logs php

# Remove and recreate
docker-compose stop php
docker-compose rm -f php
docker-compose up -d php

# Check image exists
docker images | grep devilbox-php

# Rebuild if needed
./docker-images/build-php.sh 8.4
```

### PHP Extensions Missing

```bash
# List installed extensions
docker-compose exec php php -m

# Check specific extension
docker-compose exec php php -m | grep redis

# Verify in phpinfo
open http://localhost/info_php.php
```

### Permission Issues

```bash
# Check current UID/GID
id

# Update docker-compose.override.yml
cat > docker-compose.override.yml <<EOF
services:
  php:
    image: devilbox-php-8.4:work
    environment:
      - NEW_UID=$(id -u)
      - NEW_GID=$(id -g)
EOF

# Recreate
docker-compose up -d --force-recreate php
```

## Database Connection Issues

### MySQL Won't Connect

```bash
# Check MySQL is running
docker-compose ps mysql

# Test connection from PHP container
docker-compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl -e "SELECT 1;"

# Check port forwarding
docker-compose exec php netstat -tlnp | grep 3306

# Check from host (requires socat)
mysql -h 127.0.0.1 -P 3306 -u root -proot
```

### PostgreSQL Issues

```bash
# Check PostgreSQL
docker-compose ps pgsql

# Test connection
docker-compose exec php psql -h 127.0.0.1 -U postgres -c "SELECT version();"

# Check logs
docker-compose logs pgsql
```

### Redis/Memcached Issues

```bash
# Test Redis
docker-compose exec php redis-cli -h 127.0.0.1 ping

# Test Memcached
docker-compose exec php telnet 127.0.0.1 11211
# Then type: stats
# Quit with: quit

# Check phpCacheAdmin
open http://localhost/vendor/phpcacheadmin-2.4.1/
```

## Web Server Issues

### Apache Won't Start

```bash
# Check logs
docker-compose logs httpd

# Check port conflicts
lsof -i :80
lsof -i :443

# Test configuration
docker-compose exec httpd apachectl configtest

# Restart
docker-compose restart httpd
```

### Vhost Not Working

```bash
# Check vhost config exists
ls -la data/www/my-project/.devilbox/

# View Apache vhost config
docker-compose exec httpd cat /etc/httpd/vhost.d/my-project.conf

# Check Apache logs
docker-compose exec httpd tail -f /var/log/apache2/my-project-error.log

# Force vhost regeneration
docker-compose restart httpd
```

### 502/503 Errors

```bash
# Check PHP-FPM is running
docker-compose ps php

# Check PHP-FPM status
docker-compose exec php php-fpm -t

# Check network connectivity
docker-compose exec httpd ping -c 3 php

# Verify proxy settings in vhost
docker-compose exec httpd grep "proxy:fcgi" /etc/httpd/vhost.d/*.conf
```

## Network Issues

### Containers Can't Communicate

```bash
# Check Docker network
docker network ls | grep devilbox
docker network inspect devilbox_app_net

# Check container IPs
docker inspect devilbox-php-1 | grep IPAddress
docker inspect devilbox-mysql-1 | grep IPAddress

# Test connectivity
docker-compose exec php ping -c 3 mysql
docker-compose exec php ping -c 3 redis
```

### Port Conflicts

```bash
# Find what's using port 80
sudo lsof -i :80

# Find what's using port 3306
sudo lsof -i :3306

# Change ports in .env
cat >> .env <<EOF
HOST_PORT_HTTPD=8080
HOST_PORT_MYSQL=13306
EOF

# Recreate services
docker-compose up -d
```

## Performance Issues

### Slow Container Startup

```bash
# Check disk space
df -h

# Check Docker resources
docker system df

# Clean up
docker system prune -a

# Check container resource usage
docker stats --no-stream
```

### High CPU Usage

```bash
# Monitor containers
docker stats

# Check specific processes
docker-compose exec php top

# Check for infinite loops in logs
docker-compose logs --tail=1000 php | grep -i error
```

## Complete Reset

```bash
# Stop all containers
docker-compose stop

# Remove all containers
docker-compose rm -f

# Remove custom images (optional)
docker rmi devilbox-php-8.3:work devilbox-php-8.4:work

# Rebuild images
./docker-images/build-php.sh 8.4
./docker-images/build-php.sh 8.3

# Start fresh
docker-compose up -d

# Check status
docker-compose ps
```

## Log Locations

**Inside Containers:**
- Apache: `/var/log/apache2/`
- PHP-FPM: `/var/log/php-fpm/`
- MySQL: `/var/log/mysql/`

**On Host:**
- Logs: `log/` directory
- By service: `log/php-fpm-8.4/`, `log/apache-2.4/`, etc.

## Emergency Commands

```bash
# Stop everything immediately
docker-compose kill

# Force remove all containers
docker-compose down -v

# Nuclear option (WARNING: Removes ALL Docker data)
docker system prune -a --volumes

# Rebuild everything
./docker-images/build-php.sh 8.4
docker-compose up -d
```

## Getting Help

1. Check logs first: `docker-compose logs <service>`
2. Test individual components
3. Verify configuration files
4. Check GitHub issues: https://github.com/cytopia/devilbox/issues
5. Review documentation: https://devilbox.readthedocs.io/
