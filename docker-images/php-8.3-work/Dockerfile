# Custom PHP 8.3 Work Image for Devilbox
# Based on official PHP 8.3 FPM image with essential development tools
FROM php:8.3-fpm

LABEL maintainer="Local Devilbox Development"

# Set environment variables
ENV TERM=xterm \
    COMPOSER_ALLOW_SUPERUSER=1 \
    COMPOSER_MEMORY_LIMIT=-1

# Install system dependencies
RUN apt-get update && apt-get install -y \
    # Basic utilities
    git \
    vim \
    nano \
    wget \
    curl \
    unzip \
    zip \
    rsync \
    # Build tools
    build-essential \
    # Libraries for PHP extensions
    libzip-dev \
    libpng-dev \
    libjpeg-dev \
    libfreetype6-dev \
    libonig-dev \
    libxml2-dev \
    libicu-dev \
    libpq-dev \
    libsqlite3-dev \
    libmemcached-dev \
    libssl-dev \
    zlib1g-dev \
    # Database clients
    default-mysql-client \
    postgresql-client \
    # Network tools
    netcat-traditional \
    iputils-ping \
    dnsutils \
    socat \
    # Process management
    supervisor \
    # Bash tools
    bash-completion \
    && rm -rf /var/lib/apt/lists/*

# Install PHP extensions
RUN docker-php-ext-configure gd --with-freetype --with-jpeg \
    && docker-php-ext-install -j$(nproc) \
    pdo \
    pdo_mysql \
    pdo_pgsql \
    pgsql \
    mysqli \
    mbstring \
    zip \
    exif \
    pcntl \
    bcmath \
    gd \
    intl \
    opcache \
    soap

# Install XDebug
RUN pecl install xdebug \
    && docker-php-ext-enable xdebug

# Install PECL extensions
RUN pecl install redis \
    && pecl install memcached \
    && pecl install mongodb \
    && docker-php-ext-enable redis memcached mongodb

# Install Composer
RUN curl -sS https://getcomposer.org/installer | php -- --install-dir=/usr/local/bin --filename=composer

# Install Composer global packages
RUN composer global config allow-plugins.pestphp/pest-plugin true \
    && composer global require laravel/installer \
    && composer global require wp-cli/wp-cli-bundle \
    && composer global require pestphp/pest \
    && ln -s /root/.composer/vendor/bin/laravel /usr/local/bin/laravel \
    && ln -s /root/.composer/vendor/bin/wp /usr/local/bin/wp \
    && ln -s /root/.composer/vendor/bin/pest /usr/local/bin/pest

# Install Node.js from NodeSource repository
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install global npm packages
RUN npm install -g \
    yarn \
    npm-check-updates \
    webpack \
    webpack-cli \
    grunt-cli \
    gulp-cli \
    vite \
    prettier \
    eslint \
    @vue/cli \
    @angular/cli \
    create-next-app

# Install Bun (faster alternative to Node.js)
RUN curl -fsSL https://bun.sh/install | bash \
    && ln -s /root/.bun/bin/bun /usr/local/bin/bun \
    && ln -s /root/.bun/bin/bunx /usr/local/bin/bunx

# Install sudo first (before modifying sudoers)
RUN apt-get update && apt-get install -y sudo && rm -rf /var/lib/apt/lists/*

# Create devilbox user matching host UID/GID (will be set via environment)
# Default to 1000:1000, can be overridden
ARG NEW_UID=1000
ARG NEW_GID=1000

RUN set -x \
    && if getent group ${NEW_GID} > /dev/null 2>&1; then \
        GROUP_NAME=$(getent group ${NEW_GID} | cut -d: -f1); \
        echo "GID ${NEW_GID} already exists as group '${GROUP_NAME}', using it"; \
    else \
        groupadd -g ${NEW_GID} devilbox; \
        GROUP_NAME=devilbox; \
    fi \
    && useradd -u ${NEW_UID} -g ${GROUP_NAME} -m -s /bin/bash devilbox \
    && echo "devilbox ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers \
    && ACTUAL_GROUP=$(id -gn devilbox) \
    && sed -i "s/user = www-data/user = devilbox/g" /usr/local/etc/php-fpm.d/www.conf \
    && sed -i "s/group = www-data/group = ${ACTUAL_GROUP}/g" /usr/local/etc/php-fpm.d/www.conf

# Copy entrypoint script for port forwarding
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Copy vhost auto-configuration script
COPY vhost-auto-configure.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/vhost-auto-configure.sh

# Set working directory
WORKDIR /shared/httpd

# Expose port 9000 for PHP-FPM
EXPOSE 9000

# Use entrypoint to setup port forwarding
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]

# Start PHP-FPM
CMD ["php-fpm"]
