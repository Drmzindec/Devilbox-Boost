<?php
/**
 * phpCacheAdmin configuration for Devilbox
 */

declare(strict_types=1);

return [
    'dashboards' => [
        RobiNN\Pca\Dashboards\Server\ServerDashboard::class,
        RobiNN\Pca\Dashboards\Redis\RedisDashboard::class,
        RobiNN\Pca\Dashboards\Memcached\MemcachedDashboard::class,
        RobiNN\Pca\Dashboards\OPCache\OPCacheDashboard::class,
        RobiNN\Pca\Dashboards\APCu\APCuDashboard::class,
        RobiNN\Pca\Dashboards\Realpath\RealpathDashboard::class,
    ],
    'redis' => [
        [
            'name' => 'Devilbox Redis',
            'host' => '127.0.0.1',
            'port' => 6379,
        ],
    ],
    'memcached' => [
        [
            'name' => 'Devilbox Memcached',
            'host' => '127.0.0.1',
            'port' => 11211,
        ],
    ],
    'metricsdir' => '/tmp/phpcacheadmin/metrics',
    'twigcache' => '/tmp/phpcacheadmin/twig',
];
