<?php require '../config.php'; ?>
<?php loadClass('Helper')->authPage(); ?>
<?php

/*********************************************************************************
 *
 * I N I T I A L I Z A T I O N
 *
 *********************************************************************************/


/*************************************************************
 * Get availability
 *************************************************************/
$avail_php		= loadClass('Php')->isAvailable();
$avail_dns		= loadClass('Dns')->isAvailable();
$avail_httpd	= loadClass('Httpd')->isAvailable();
$avail_mysql	= loadClass('Mysql')->isAvailable();
$avail_pgsql	= loadClass('Pgsql')->isAvailable();
$avail_redis	= loadClass('Redis')->isAvailable();
$avail_memcd	= loadClass('Memcd')->isAvailable();
$avail_mongo	= loadClass('Mongo')->isAvailable();


/*************************************************************
 * Test Connectivity
 *************************************************************/

$connection = array();
$error	= null;

// ---- HTTPD (required) ----

$host	= $GLOBALS['HTTPD_HOST_NAME'];
$succ	= loadClass('Httpd')->canConnect($error, $host);
$connection['Httpd'][$host] = array(
	'error' => $error,
	'host' => $host,
	'succ' => $succ
);
$host	= loadClass('Httpd')->getIpAddress();
$succ	= loadClass('Httpd')->canConnect($error, $host);
$connection['Httpd'][$host] = array(
	'error' => $error,
	'host' => $host,
	'succ' => $succ
);
// ---- MYSQL ----
if ($avail_mysql) {
	$host	= $GLOBALS['MYSQL_HOST_NAME'];
	$succ	= loadClass('Mysql')->canConnect($error, $host, array('user' => 'root', 'pass' => loadClass('Helper')->getEnv('MYSQL_ROOT_PASSWORD')));
	$connection['MySQL'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
	$host	= loadClass('Mysql')->getIpAddress();
	$succ	= loadClass('Mysql')->canConnect($error, $host, array('user' => 'root', 'pass' => loadClass('Helper')->getEnv('MYSQL_ROOT_PASSWORD')));
	$connection['MySQL'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
	$host	= '127.0.0.1';
	$succ	= loadClass('Mysql')->canConnect($error, $host, array('user' => 'root', 'pass' => loadClass('Helper')->getEnv('MYSQL_ROOT_PASSWORD')));
	$connection['MySQL'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
}

// ---- PGSQL ----
if ($avail_pgsql) {
	$host	= $GLOBALS['PGSQL_HOST_NAME'];
	$succ	= loadClass('Pgsql')->canConnect($error, $host, array('user' => loadClass('Helper')->getEnv('PGSQL_ROOT_USER'), 'pass' => loadClass('Helper')->getEnv('PGSQL_ROOT_PASSWORD')));
	$connection['PgSQL'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
	$host	= loadClass('Pgsql')->getIpAddress();
	$succ	= loadClass('Pgsql')->canConnect($error, $host, array('user' => loadClass('Helper')->getEnv('PGSQL_ROOT_USER'), 'pass' => loadClass('Helper')->getEnv('PGSQL_ROOT_PASSWORD')));
	$connection['PgSQL'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
	$host	= '127.0.0.1';
	$succ	= loadClass('Pgsql')->canConnect($error, $host, array('user' => loadClass('Helper')->getEnv('PGSQL_ROOT_USER'), 'pass' => loadClass('Helper')->getEnv('PGSQL_ROOT_PASSWORD')));
	$connection['PgSQL'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
}

// ---- REDIS ----
if ($avail_redis) {
	$host	= $GLOBALS['REDIS_HOST_NAME'];
	$succ	= loadClass('Redis')->canConnect($error, $host);
	$connection['Redis'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
	$host	= loadClass('Redis')->getIpAddress();
	$succ	= loadClass('Redis')->canConnect($error, $host);
	$connection['Redis'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
	$host	= '127.0.0.1';
	$succ	= loadClass('Redis')->canConnect($error, $host);
	$connection['Redis'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
}


// ---- MEMCACHED ----
if ($avail_memcd) {
	$host	= $GLOBALS['MEMCD_HOST_NAME'];
	$succ	= loadClass('Memcd')->canConnect($error, $host);
	$connection['Memcached'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
	$host	= loadClass('Memcd')->getIpAddress();
	$succ	= loadClass('Memcd')->canConnect($error, $host);
	$connection['Memcached'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
	$host	= '127.0.0.1';
	$succ	= loadClass('Memcd')->canConnect($error, $host);
	$connection['Memcached'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
}

// ---- MONGO ----
if ($avail_mongo) {
	$host	= $GLOBALS['MONGO_HOST_NAME'];
	$succ	= loadClass('Mongo')->canConnect($error, $host);
	$connection['MongoDB'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
	$host	= loadClass('Mongo')->getIpAddress();
	$succ	= loadClass('Mongo')->canConnect($error, $host);
	$connection['MongoDB'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
	$host	= '127.0.0.1';
	$succ	= loadClass('Mongo')->canConnect($error, $host);
	$connection['MongoDB'][$host] = array(
		'error' => $error,
		'host' => $host,
		'succ' => $succ
	);
}


// ---- BIND (required)----
$host	= $GLOBALS['DNS_HOST_NAME'];
$succ	= loadClass('Dns')->canConnect($error, $host);
$connection['Bind'][$host] = array(
	'error' => $error,
	'host' => $host,
	'succ' => $succ
);
$host	= loadClass('Dns')->getIpAddress();
$succ	= loadClass('Dns')->canConnect($error, $host);
$connection['Bind'][$host] = array(
	'error' => $error,
	'host' => $host,
	'succ' => $succ
);


/*************************************************************
 * Test Health
 *************************************************************/
$HEALTH_TOTAL = 0;
$HEALTH_FAILS = 0;

foreach ($connection as $docker) {
	foreach ($docker as $conn) {
		if (!$conn['succ']) {
			$HEALTH_FAILS++;
		}
		$HEALTH_TOTAL++;
	}
}
$HEALTH_PERCENT = 100 - ceil(100 * $HEALTH_FAILS / $HEALTH_TOTAL);


/*************************************************************
 * Check Modern Services
 *************************************************************/
function checkServicePort($host, $port, $timeout = 1) {
	$fp = @fsockopen($host, $port, $errno, $errstr, $timeout);
	if ($fp) {
		fclose($fp);
		return true;
	}
	return false;
}

$modern_services = array(
	'meilisearch' => array(
		'name' => 'Meilisearch',
		'port' => 7700,
		'url' => 'http://localhost:7700',
		'master_key' => loadClass('Helper')->getEnv('MEILI_MASTER_KEY') ?: 'masterKey',
		'config' => array(
			'Laravel Scout' => "SCOUT_DRIVER=meilisearch\nMEILISEARCH_HOST=http://127.0.0.1:7700\nMEILISEARCH_KEY=masterKey",
			'PHP' => "\$client = new \\MeiliSearch\\Client('http://127.0.0.1:7700', 'masterKey');"
		)
	),
	'mailpit' => array(
		'name' => 'Mailpit',
		'port' => 8025,
		'url' => 'http://localhost:8025',
		'smtp' => array(
			'host' => '127.0.0.1',
			'port' => 1025,
			'auth' => false
		),
		'config' => array(
			'Laravel' => "MAIL_MAILER=smtp\nMAIL_HOST=127.0.0.1\nMAIL_PORT=1025\nMAIL_USERNAME=null\nMAIL_PASSWORD=null\nMAIL_ENCRYPTION=null",
			'WordPress' => "define('SMTP_HOST', '127.0.0.1');\ndefine('SMTP_PORT', 1025);\ndefine('SMTP_AUTH', false);"
		)
	),
	'rabbit' => array(
		'name' => 'RabbitMQ',
		'port' => 15672,
		'url' => 'http://localhost:15672',
		'username' => loadClass('Helper')->getEnv('RABBIT_DEFAULT_USER') ?: 'guest',
		'password' => loadClass('Helper')->getEnv('RABBIT_DEFAULT_PASS') ?: 'guest',
		'amqp_port' => 5672,
		'config' => array(
			'Laravel' => "QUEUE_CONNECTION=rabbitmq\nRABBITMQ_HOST=127.0.0.1\nRABBITMQ_PORT=5672\nRABBITMQ_USER=guest\nRABBITMQ_PASSWORD=guest",
			'PHP' => "\$connection = new \\PhpAmqpLib\\Connection\\AMQPStreamConnection('127.0.0.1', 5672, 'guest', 'guest');"
		)
	),
	'minio' => array(
		'name' => 'MinIO',
		'port' => 9001,
		'url' => 'http://localhost:9001',
		'username' => loadClass('Helper')->getEnv('MINIO_ROOT_USER') ?: 'minioadmin',
		'password' => loadClass('Helper')->getEnv('MINIO_ROOT_PASSWORD') ?: 'minioadmin',
		'api_port' => 9000,
		'config' => array(
			'Laravel' => "AWS_ACCESS_KEY_ID=minioadmin\nAWS_SECRET_ACCESS_KEY=minioadmin\nAWS_DEFAULT_REGION=us-east-1\nAWS_BUCKET=my-bucket\nAWS_ENDPOINT=http://127.0.0.1:9000\nAWS_USE_PATH_STYLE_ENDPOINT=true",
			'PHP' => "\$client = new \\Aws\\S3\\S3Client([\n  'endpoint' => 'http://127.0.0.1:9000',\n  'credentials' => ['key' => 'minioadmin', 'secret' => 'minioadmin']\n]);"
		)
	)
);

// Check which services are running
foreach ($modern_services as $key => &$service) {
	$service['running'] = checkServicePort('127.0.0.1', $service['port']);
}


/*********************************************************************************
 *
 * H T M L
 *
 *********************************************************************************/
?>
<!DOCTYPE html>
<html lang="en">
	<head>
		<?php echo loadClass('Html')->getHead(true); ?>
	</head>

	<body style="background: #1f1f1f;">
		<?php echo loadClass('Html')->getNavbar(); ?>


		<div class="container">


			<!-- ############################################################ -->
			<!-- Version/Health -->
			<!-- ############################################################ -->
			<div class="row">

				<div class="col-md-4 col-sm-4 col-xs-12 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-hashtag"></i> Version</div>
						<div class="dash-box-body">
							<strong>Devilbox</strong> <?php echo $GLOBALS['DEVILBOX_VERSION']; ?> <small>(<?php echo $GLOBALS['DEVILBOX_DATE']; ?>)</small>
						</div>
					</div>
				</div>

				<div class="col-md-4 col-sm-4 col-xs-12 col-margin">
					<img src="/assets/img/banner.png" style="width:100%;" />
				</div>

				<div class="col-md-4 col-sm-4 col-xs-12 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-bug" aria-hidden="true"></i> Health</div>
						<div class="dash-box-body">
							<div class="meter">
							  <span style="color:black; width: <?php echo $HEALTH_PERCENT; ?>%">&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<?php echo $HEALTH_PERCENT; ?>%</span>
							</div>
						</div>
					</div>
				</div>

			</div><!-- /row -->



			<!-- ############################################################ -->
			<!-- DASH -->
			<!-- ############################################################ -->
			<div class="row">

				<div class="col-md-4 col-sm-4 col-xs-12 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-cog" aria-hidden="true"></i> Base Stack</div>
						<div class="dash-box-body">
							<div class="row">
								<div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 col-xs-4" style="margin-bottom:15px;">
									<?php echo loadClass('Html')->getCirle('dns'); ?>
								</div>
								<div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 col-xs-4" style="margin-bottom:15px;">
									<?php echo loadClass('Html')->getCirle('php'); ?>
								</div>
								<div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 col-xs-4" style="margin-bottom:15px;">
									<?php echo loadClass('Html')->getCirle('httpd'); ?>
								</div>
							</div>
						</div>
					</div>
				</div>

				<div class="col-md-4 col-sm-4 col-xs-12 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-database" aria-hidden="true"></i> SQL Stack</div>
						<div class="dash-box-body">
							<div class="row">
								<div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 col-xs-4" style="margin-bottom:15px;">
									<?php echo loadClass('Html')->getCirle('mysql'); ?>
								</div>
								<div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 col-xs-4" style="margin-bottom:15px;">
									<?php echo loadClass('Html')->getCirle('pgsql'); ?>
								</div>
							</div>
						</div>
					</div>
				</div>

				<div class="col-md-4 col-sm-4 col-xs-12 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-file-o" aria-hidden="true"></i> NoSQL Stack</div>
						<div class="dash-box-body">
							<div class="row">
								<div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 col-xs-4" style="margin-bottom:15px;">
									<?php echo loadClass('Html')->getCirle('redis'); ?>
								</div>
								<div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 col-xs-4" style="margin-bottom:15px;">
									<?php echo loadClass('Html')->getCirle('memcd'); ?>
								</div>
								<div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 col-xs-4" style="margin-bottom:15px;">
									<?php echo loadClass('Html')->getCirle('mongo'); ?>
								</div>
							</div>
						</div>
					</div>
				</div>

			</div><!-- /row -->


			<!-- ############################################################ -->
			<!-- MODERN SERVICES -->
			<!-- ############################################################ -->
			<div class="row">
				<div class="col-md-12 col-sm-12 col-xs-12 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-rocket" aria-hidden="true"></i> Modern Services (Optional)</div>
						<div class="dash-box-body">

							<div class="row">
								<?php foreach ($modern_services as $key => $service): ?>
								<div class="col-xl-3 col-lg-6 col-md-6 col-sm-12" style="margin-bottom:20px;">
									<div class="card text-white <?php echo $service['running'] ? 'bg-success' : 'bg-secondary'; ?>" style="border-radius:8px; height:100%;">
										<div class="card-body" style="padding:15px;">
											<h5 class="card-title">
												<?php if ($key == 'meilisearch'): ?>
													<i class="fa fa-search"></i> <?php echo $service['name']; ?>
												<?php elseif ($key == 'mailpit'): ?>
													<i class="fa fa-envelope"></i> <?php echo $service['name']; ?>
												<?php elseif ($key == 'rabbit'): ?>
													<i class="fa fa-exchange"></i> <?php echo $service['name']; ?>
												<?php elseif ($key == 'minio'): ?>
													<i class="fa fa-database"></i> <?php echo $service['name']; ?>
												<?php endif; ?>
												<?php if ($service['running']): ?>
													<span class="badge badge-light" style="font-size:10px; float:right;">RUNNING</span>
												<?php else: ?>
													<span class="badge badge-dark" style="font-size:10px; float:right;">STOPPED</span>
												<?php endif; ?>
											</h5>

											<p class="card-text"><small>
												<?php if ($key == 'meilisearch'): ?>
													Fast search engine
												<?php elseif ($key == 'mailpit'): ?>
													Email testing
												<?php elseif ($key == 'rabbit'): ?>
													Message queue
												<?php elseif ($key == 'minio'): ?>
													S3 storage
												<?php endif; ?>
											</small></p>

											<?php if ($service['running']): ?>
												<a href="<?php echo $service['url']; ?>" target="_blank" class="btn btn-light btn-sm btn-block" style="margin-bottom:10px;">
													<i class="fa fa-external-link"></i> Open Dashboard
												</a>
											<?php else: ?>
												<button class="btn btn-dark btn-sm btn-block" disabled style="margin-bottom:10px;">
													<i class="fa fa-power-off"></i> Not Running
												</button>
											<?php endif; ?>

											<div style="background:#f8f9fa; color:#333; padding:8px; border-radius:4px; font-size:11px;">
												<?php if ($key == 'meilisearch'): ?>
													<strong>Master Key:</strong><br/>
													<code style="background:#fff; padding:2px 4px; color:#d63384;"><?php echo $service['master_key']; ?></code>
												<?php elseif ($key == 'mailpit'): ?>
													<strong>SMTP:</strong> <?php echo $service['smtp']['host']; ?>:<?php echo $service['smtp']['port']; ?><br/>
													<small>No authentication required</small>
												<?php elseif ($key == 'rabbit'): ?>
													<strong>User:</strong> <code style="background:#fff; padding:2px 4px;"><?php echo $service['username']; ?></code><br/>
													<strong>Pass:</strong> <code style="background:#fff; padding:2px 4px;"><?php echo $service['password']; ?></code><br/>
													<small>AMQP Port: <?php echo $service['amqp_port']; ?></small>
												<?php elseif ($key == 'minio'): ?>
													<strong>User:</strong> <code style="background:#fff; padding:2px 4px;"><?php echo $service['username']; ?></code><br/>
													<strong>Pass:</strong> <code style="background:#fff; padding:2px 4px;"><?php echo $service['password']; ?></code><br/>
													<small>API Port: <?php echo $service['api_port']; ?></small>
												<?php endif; ?>
											</div>
										</div>
									</div>
								</div>
								<?php endforeach; ?>
							</div>

							<div class="row" style="margin-top:15px;">
								<div class="col-12">
									<h6 style="color:#9ccc65;"><i class="fa fa-code"></i> Quick Start</h6>
									<ul class="nav nav-tabs" id="servicesTabs" role="tablist">
										<li class="nav-item">
											<a class="nav-link active" id="meilisearch-tab" data-toggle="tab" href="#meilisearch-config" role="tab">Meilisearch</a>
										</li>
										<li class="nav-item">
											<a class="nav-link" id="mailpit-tab" data-toggle="tab" href="#mailpit-config" role="tab">Mailpit</a>
										</li>
										<li class="nav-item">
											<a class="nav-link" id="rabbitmq-tab" data-toggle="tab" href="#rabbitmq-config" role="tab">RabbitMQ</a>
										</li>
										<li class="nav-item">
											<a class="nav-link" id="minio-tab" data-toggle="tab" href="#minio-config" role="tab">MinIO</a>
										</li>
									</ul>
									<div class="tab-content" id="servicesTabContent" style="background:#2a2a2a; padding:15px; border-radius:0 0 4px 4px;">
										<div class="tab-pane fade show active" id="meilisearch-config" role="tabpanel">
											<h6>Laravel Scout Configuration</h6>
											<pre style="background:#1a1a1a; color:#9ccc65; padding:10px; border-radius:4px; font-size:11px;"><?php echo htmlspecialchars($modern_services['meilisearch']['config']['Laravel Scout']); ?></pre>
										</div>
										<div class="tab-pane fade" id="mailpit-config" role="tabpanel">
											<h6>Laravel Mail Configuration</h6>
											<pre style="background:#1a1a1a; color:#9ccc65; padding:10px; border-radius:4px; font-size:11px;"><?php echo htmlspecialchars($modern_services['mailpit']['config']['Laravel']); ?></pre>
											<h6 style="margin-top:15px;">WordPress SMTP</h6>
											<pre style="background:#1a1a1a; color:#9ccc65; padding:10px; border-radius:4px; font-size:11px;"><?php echo htmlspecialchars($modern_services['mailpit']['config']['WordPress']); ?></pre>
										</div>
										<div class="tab-pane fade" id="rabbitmq-config" role="tabpanel">
											<h6>Laravel Queue Configuration</h6>
											<pre style="background:#1a1a1a; color:#9ccc65; padding:10px; border-radius:4px; font-size:11px;"><?php echo htmlspecialchars($modern_services['rabbit']['config']['Laravel']); ?></pre>
										</div>
										<div class="tab-pane fade" id="minio-config" role="tabpanel">
											<h6>Laravel S3 Configuration</h6>
											<pre style="background:#1a1a1a; color:#9ccc65; padding:10px; border-radius:4px; font-size:11px;"><?php echo htmlspecialchars($modern_services['minio']['config']['Laravel']); ?></pre>
										</div>
									</div>
									<div style="margin-top:10px;">
										<small class="text-muted">
											<i class="fa fa-info-circle"></i>
											Need more examples? See <a href="https://github.com/Drmzindec/Devilbox-Boost/blob/main/docs/MODERN-SERVICES.md" target="_blank">full documentation</a> with PHP, WordPress, and framework-specific guides.
										</small>
									</div>
								</div>
							</div>

						</div>
					</div>
				</div>
			</div><!-- /row -->


			<!-- ############################################################ -->
			<!-- Settings / Status -->
			<!-- ############################################################ -->

			<div class="row">
				<div class="col-lg-4 col-md-6 col-sm-12 col-xs-12 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-terminal" aria-hidden="true"></i> Available CLI Tools</div>
						<div class="dash-box-body">
							<p><small>You can also enter the php container via <code style="background-color:#3d3d3d;">./shell.sh</code> and use the following cli tools:</small></p>
							<table class="table table-striped table-hover table-bordered table-sm font-small" style="margin-bottom: 0;">
								<thead class="thead-inverse">
									<tr>
										<th colspan="2">Tools</th>
									</tr>
								</thead>
								<tbody>
									<!-- Core Tools -->
									<tr style="background-color: #2a2a2a;">
										<th colspan="2" style="color: #9ccc65; font-weight: bold; padding: 8px;"><i class="fa fa-cogs" aria-hidden="true"></i> Core & PHP</th>
									</tr>
									<tr>
										<th style="width: 50%; padding-left: 20px;">Git</th>
										<td id="app_git"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Composer</th>
										<td id="app_composer"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Laravel Installer</th>
										<td id="app_laravel_installer"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">WordPress CLI</th>
										<td id="app_wpcli"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Pest (Testing)</th>
										<td id="app_pest"></td>
									</tr>
									<!-- JavaScript & Runtimes -->
									<tr style="background-color: #2a2a2a;">
										<th colspan="2" style="color: #ffc107; font-weight: bold; padding: 8px;"><i class="fa fa-code" aria-hidden="true"></i> JavaScript & Build</th>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Node.js</th>
										<td id="app_node"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Bun</th>
										<td id="app_bun"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">NPM</th>
										<td id="app_npm"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Yarn</th>
										<td id="app_yarn"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Vite</th>
										<td id="app_vite"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Webpack</th>
										<td id="app_webpack_cli"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Gulp</th>
										<td id="app_gulp"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Grunt</th>
										<td id="app_grunt_cli"></td>
									</tr>
									<!-- Code Quality & Frameworks -->
									<tr style="background-color: #2a2a2a;">
										<th colspan="2" style="color: #64b5f6; font-weight: bold; padding: 8px;"><i class="fa fa-check-circle" aria-hidden="true"></i> Quality & Frameworks</th>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Prettier</th>
										<td id="app_prettier"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">ESLint</th>
										<td id="app_eslint"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Vue CLI</th>
										<td id="app_vue_cli"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">React CLI</th>
										<td id="app_react_cli"></td>
									</tr>
									<tr>
										<th style="padding-left: 20px;">Angular CLI</th>
										<td id="app_angular_cli"></td>
									</tr>
								</tbody>
							</table>

						</div>
					</div>
				</div>

				<div class="col-lg-4 col-md-6 col-sm-12 col-xs-12 offset-lg-4 offset-md-0 offset-sm-0 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-info-circle" aria-hidden="true"></i> PHP Container Status</div>
						<div class="dash-box-body">
							<p><small>You have made the following base configuration to the Devilbox:</small></p>
							<table class="table table-striped table-hover table-bordered table-sm font-small">
								<thead class="thead-inverse">
									<tr>
										<th colspan="2">Settings</th>
									</tr>
								</thead>
								<tbody>
									<tr>
										<th>uid</th>
										<td><?php echo loadClass('Php')->getUid(); ?></td>
									</tr>
									<tr>
										<th>gid</th>
										<td><?php echo loadClass('Php')->getGid(); ?></td>
									</tr>
									<tr>
										<th>vHost docroot dir</th>
										<td><?php echo loadClass('Helper')->getEnv('HTTPD_DOCROOT_DIR') ?: '/ (vhost root)'; ?></td>
									</tr>
									<tr>
										<th>vHost config dir</th>
										<td><?php echo loadClass('Helper')->getEnv('HTTPD_TEMPLATE_DIR'); ?></td>
									</tr>
									<tr>
										<th>vHost TLD</th>
										<td>*.<?php echo loadClass('Httpd')->getTldSuffix(); ?></td>
									</tr>
									<tr>
										<th>DNS</th>
										<td><?php if ($avail_dns): ?>Enabled<?php else: ?><span class="text-danger">Offline</span><?php endif;?></td>
									</tr>
									<tr>
										<th>Postfix</th>
										<td><?php echo loadClass('Helper')->getEnv('ENABLE_MAIL') ? 'Enabled'  : '<span class="bg-danger">No</span> Disabled';?></td>
									</tr>
								</tbody>
							</table>
							<p><small>The PHP container can connect to the following services via the specified hostnames and IP addresses.</small></p>
							<table class="table table-striped table-hover table-bordered table-sm font-small">
								<thead class="thead-inverse">
									<tr>
										<th>Service</th>
										<th>Hostname / IP</th>
									</tr>
								</thead>
								<tbody>
									<?php foreach ($connection as $name => $docker): ?>
										<tr>
											<th rowspan="<?php echo count($docker);?>" class="align-middle"><?php echo $name; ?> connect</th>
											<?php $i=1; foreach ($docker as $conn): ?>

											<?php if ($conn['succ']): ?>
												<?php $text = '<span class="text-success dvlbox-ok"><i class="fa fa-check-square"></i></span> '.$conn['host']; ?>
											<?php else: ?>
												<?php $text = '<span class="text-danger dvlbox-err"><i class="fa fa-exclamation-triangle"></i></span> '.$conn['host'].'<br/>'.$conn['error']; ?>
											<?php endif; ?>

												<?php if ($i == 1): $i++;?>
													<td>
														<?php echo $text; ?>
													</td>
													</tr>
												<?php else: $i++;?>
													<tr>
														<td>
															<?php echo $text; ?>
														</td>
													</tr>
												<?php endif; ?>
											<?php endforeach; ?>
									<?php endforeach; ?>
								</tbody>
							</table>
						</div>
					</div>
				</div>


			</div><!-- /row -->


			<!-- ############################################################ -->
			<!-- TABLES -->
			<!-- ############################################################ -->
			<div class="row">

				<div class="col-lg-4 col-md-6 col-sm-12 col-xs-12 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-share-alt" aria-hidden="true"></i> Networking</div>
						<div class="dash-box-body">
							<div class="row">
								<div class="container">
									<table class="table table-striped table-hover table-bordered table-sm font-small">
										<thead class="thead-inverse">
											<tr>
												<th>Docker</th>
												<th>Hostname</th>
												<th>IP</th>
											</tr>
										</thead>
										<tbody>
											<tr>
												<th>php</th>
												<td><?php echo $GLOBALS['PHP_HOST_NAME']; ?></td>
												<td><?php echo loadClass('Php')->getIpAddress(); ?></td>
											</tr>
											<tr>
												<th>httpd</th>
												<td><?php echo $GLOBALS['HTTPD_HOST_NAME']; ?></td>
												<td><?php echo loadClass('Httpd')->getIpAddress(); ?></td>
											</tr>
											<?php if ($avail_mysql): ?>
												<tr>
													<th>mysql</th>
													<td><?php echo $GLOBALS['MYSQL_HOST_NAME']; ?></td>
													<td><?php echo loadClass('Mysql')->getIpAddress(); ?></td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_pgsql): ?>
												<tr>
													<th>pgsql</th>
													<td><?php echo $GLOBALS['PGSQL_HOST_NAME']; ?></td>
													<td><?php echo loadClass('Pgsql')->getIpAddress(); ?></td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_redis): ?>
												<tr>
													<th>redis</th>
													<td><?php echo $GLOBALS['REDIS_HOST_NAME']; ?></td>
													<td><?php echo loadClass('Redis')->getIpAddress(); ?></td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_memcd): ?>
												<tr>
													<th>memcached</th>
													<td><?php echo $GLOBALS['MEMCD_HOST_NAME']; ?></td>
													<td><?php echo loadClass('Memcd')->getIpAddress(); ?></td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_mongo): ?>
												<tr>
													<th>mongo</th>
													<td><?php echo $GLOBALS['MONGO_HOST_NAME']; ?></td>
													<td><?php echo loadClass('Mongo')->getIpAddress(); ?></td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_dns): ?>
												<tr>
													<th>bind</th>
													<td><?php echo $GLOBALS['DNS_HOST_NAME']; ?></td>
													<td><?php echo loadClass('Dns')->getIpAddress(); ?></td>
												</tr>
											<?php endif; ?>
										</tbody>
									</table>
								</div>
							</div>
						</div>
					</div>
				</div>


				<div class="col-lg-4 col-md-6 col-sm-12 col-xs-12 offset-lg-4 offset-md-0 offset-sm-0 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-wrench" aria-hidden="true"></i> Ports</div>
						<div class="dash-box-body">
							<div class="row">
								<div class="container">
									<table class="table table-striped table-hover table-bordered table-sm font-small">
										<thead class="thead-inverse">
											<tr>
												<th>Docker</th>
												<th>Host port</th>
												<th>Docker port</th>
											</tr>
										</thead>
										<tbody>
											<tr>
												<th>php</th>
												<td>-</td>
												<td>9000</td>
											</tr>
											<tr>
												<th>httpd</th>
												<td>
													<?php echo loadClass('Helper')->getEnv('LOCAL_LISTEN_ADDR').loadClass('Helper')->getEnv('HOST_PORT_HTTPD');?><br/>
													<?php echo loadClass('Helper')->getEnv('LOCAL_LISTEN_ADDR').loadClass('Helper')->getEnv('HOST_PORT_HTTPD_SSL');?>
												</td>
												<td>80<br/>443</td>
											</tr>
											<?php if ($avail_mysql): ?>
												<tr>
													<th>mysql</th>
													<td><?php echo loadClass('Helper')->getEnv('LOCAL_LISTEN_ADDR').loadClass('Helper')->getEnv('HOST_PORT_MYSQL');?></td>
													<td>3306</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_pgsql): ?>
												<tr>
													<th>pgsql</th>
													<td><?php echo loadClass('Helper')->getEnv('LOCAL_LISTEN_ADDR').loadClass('Helper')->getEnv('HOST_PORT_PGSQL');?></td>
													<td>5432</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_redis): ?>
												<tr>
													<th>redis</th>
													<td><?php echo loadClass('Helper')->getEnv('LOCAL_LISTEN_ADDR').loadClass('Helper')->getEnv('HOST_PORT_REDIS');?></td>
													<td>6379</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_memcd): ?>
												<tr>
													<th>memcached</th>
													<td><?php echo loadClass('Helper')->getEnv('LOCAL_LISTEN_ADDR').loadClass('Helper')->getEnv('HOST_PORT_MEMCD');?></td>
													<td>11211</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_mongo): ?>
												<tr>
													<th>mongo</th>
													<td><?php echo loadClass('Helper')->getEnv('LOCAL_LISTEN_ADDR').loadClass('Helper')->getEnv('HOST_PORT_MONGO');?></td>
													<td>27017</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_dns): ?>
												<tr>
													<th>bind</th>
													<td>
														<?php echo loadClass('Helper')->getEnv('LOCAL_LISTEN_ADDR').loadClass('Helper')->getEnv('HOST_PORT_BIND');?>/tcp<br/>
														<?php echo loadClass('Helper')->getEnv('LOCAL_LISTEN_ADDR').loadClass('Helper')->getEnv('HOST_PORT_BIND');?>/udp
														</td>
													<td>53/tcp<br/>53/udp</td>
												</tr>
											<?php endif; ?>
										</tbody>
									</table>
								</div>
							</div>
						</div>
					</div>
				</div>

				<div class="col-lg-4 col-md-6 col-sm-12 col-xs-12 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-hdd-o" aria-hidden="true"></i> Data mounts</div>
						<div class="dash-box-body">
							<div class="row">
								<div class="container">
									<table class="table table-striped table-hover table-bordered table-sm font-small" style="word-break: break-word;">
										<thead class="thead-inverse">
											<tr>
												<th>Docker</th>
												<th>Host path</th>
												<th>Docker path</th>
											</tr>
										</thead>
										<tbody>
											<tr>
												<th>php</th>
													<td><?php echo loadClass('Helper')->getEnv('HOST_PATH_HTTPD_DATADIR'); ?></td>
												<td>/shared/httpd</td>
											</tr>
											<tr>
												<th>httpd</th>
													<td><?php echo loadClass('Helper')->getEnv('HOST_PATH_HTTPD_DATADIR'); ?></td>
												<td>/shared/httpd</td>
											</tr>
											<?php if ($avail_mysql): ?>
												<tr>
													<th>mysql</th>
													<td>Docker volume</td>
													<td>/var/lib/mysql</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_pgsql): ?>
												<tr>
													<th>pgsql</th>
													<td>Docker volume</td>
													<td>/var/lib/postgresql/data/pgdata</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_redis): ?>
												<tr>
													<th>redis</th>
													<td>-</td>
													<td>-</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_memcd): ?>
												<tr>
													<th>memcached</th>
													<td>-</td>
													<td>-</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_mongo): ?>
												<tr>
													<th>mongo</th>
													<td>Docker volume</td>
													<td>/data/db</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_dns): ?>
												<tr>
													<th>bind</th>
													<td>-</td>
													<td>-</td>
												</tr>
											<?php endif; ?>
										</tbody>
									</table>
								</div>
							</div>
						</div>
					</div>
				</div>

				<div class="col-lg-4 col-md-6 col-sm-12 col-xs-12 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-cogs" aria-hidden="true"></i> Config mounts</div>
						<div class="dash-box-body">
							<div class="row">
								<div class="container">
									<table class="table table-striped table-hover table-bordered table-sm font-small">
										<thead class="thead-inverse">
											<tr>
												<th>Docker</th>
												<th>Host path</th>
												<th>Docker path</th>
											</tr>
										</thead>
										<tbody>
											<tr>
												<th>php (ini)</th>
												<td>./cfg/php-ini-<?php echo loadClass('Helper')->getEnv('PHP_SERVER'); ?></td>
												<td>/etc/php-custom.d</td>
											</tr>
											<tr>
												<th>php (fpm)</th>
												<td>./cfg/php-fpm-<?php echo loadClass('Helper')->getEnv('PHP_SERVER'); ?></td>
												<td>/etc/php-fpm-custom.d</td>
											</tr>
											<tr>
												<th>httpd</th>
												<td>./cfg/<?php echo loadClass('Helper')->getEnv('HTTPD_SERVER'); ?></td>
												<td>/etc/httpd-custom.d</td>
											</tr>
											<?php if ($avail_mysql): ?>
												<tr>
													<th>mysql</th>
													<td>./cfg/<?php echo loadClass('Helper')->getEnv('MYSQL_SERVER'); ?></td>
													<td>/etc/mysql/conf.d</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_pgsql): ?>
												<tr>
													<th>pgsql</th>
													<td>-</td>
													<td>-</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_redis): ?>
												<tr>
													<th>redis</th>
													<td>-</td>
													<td>-</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_memcd): ?>
												<tr>
													<th>memcached</th>
													<td>-</td>
													<td>-</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_mongo): ?>
												<tr>
													<th>mongo</th>
													<td>-</td>
													<td>-</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_dns): ?>
												<tr>
													<th>bind</th>
													<td>-</td>
													<td>-</td>
												</tr>
											<?php endif; ?>
										</tbody>
									</table>
								</div>
							</div>
						</div>
					</div>
				</div>

				<div class="col-lg-4 col-md-6 col-sm-12 col-xs-12 col-margin">
					<div class="dash-box">
						<div class="dash-box-head"><i class="fa fa-bar-chart" aria-hidden="true"></i> Log mounts</div>
						<div class="dash-box-body">
							<div class="row">
								<div class="container">
									<table class="table table-striped table-hover table-bordered table-sm font-small" style="word-break: break-word;">
										<thead class="thead-inverse">
											<tr>
												<th>Docker</th>
												<th>Host path</th>
												<th>Docker path</th>
											</tr>
										</thead>
										<tbody>
											<tr>
												<th>php</th>
												<td>./log/php-fpm-<?php echo loadClass('Helper')->getEnv('PHP_SERVER'); ?></td>
												<td>/var/log/php</td>
											</tr>
											<tr>
												<th>httpd</th>
												<td>./log/<?php echo loadClass('Helper')->getEnv('HTTPD_SERVER'); ?></td>
												<td>/var/log/<?php echo loadClass('Helper')->getEnv('HTTPD_SERVER'); ?></td>
											</tr>
											<?php if ($avail_mysql): ?>
												<tr>
													<th>mysql</th>
													<td>./log/<?php echo loadClass('Helper')->getEnv('MYSQL_SERVER'); ?></td>
													<td>/var/log/mysql</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_pgsql): ?>
												<tr>
													<th>pgsql</th>
													<td>./log/pgsql-<?php echo loadClass('Helper')->getEnv('PGSQL_SERVER'); ?></td>
													<td>/var/log/postgresql</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_redis): ?>
												<tr>
													<th>redis</th>
													<td>./log/redis-<?php echo loadClass('Helper')->getEnv('REDIS_SERVER'); ?></td>
													<td>/var/log/redis</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_memcd): ?>
												<tr>
													<th>memcached</th>
													<td>./log/memcached-<?php echo loadClass('Helper')->getEnv('MEMCD_SERVER'); ?></td>
													<td>/var/log/memcached</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_mongo): ?>
												<tr>
													<th>mongo</th>
													<td>-</td>
													<td>-</td>
												</tr>
											<?php endif; ?>
											<?php if ($avail_dns): ?>
												<tr>
													<th>bind</th>
													<td>-</td>
													<td>-</td>
												</tr>
											<?php endif; ?>
										</tbody>
									</table>
								</div>
							</div>
						</div>
					</div>
				</div>


			</div><!-- /row -->


		</div><!-- /.container -->

		<?php echo loadClass('Html')->getFooter(); ?>
		<script>
		// self executing function here
		(function() {
			// your page initialization code here
			// the DOM will be available here

			/**
			 * Update installed tool versions.
			 * Ajax method is faster for loading the front page
			 * @param  string app Name of the tool
			 */
			function updateVersions(app) {
				var xhttp = new XMLHttpRequest();

				xhttp.onreadystatechange = function() {
					var elem = document.getElementById('app_'+app);

					if (this.readyState == 4 && this.status == 200) {
						json = JSON.parse(this.responseText);
						elem.innerHTML = json[app];
					}
				};
				xhttp.open('GET', '_ajax_callback.php?software='+app, true);
				xhttp.send();
			}
			// Core Tools
			updateVersions('git');
			updateVersions('composer');
			// PHP Tools
			updateVersions('laravel_installer');
			updateVersions('wpcli');
			updateVersions('pest');
			// JS Runtimes
			updateVersions('node');
			updateVersions('bun');
			// Package Managers
			updateVersions('npm');
			updateVersions('yarn');
			// Build Tools
			updateVersions('vite');
			updateVersions('webpack_cli');
			updateVersions('gulp');
			updateVersions('grunt_cli');
			// Code Quality
			updateVersions('prettier');
			updateVersions('eslint');
			// Framework CLIs
			updateVersions('vue_cli');
			updateVersions('react_cli');
			updateVersions('angular_cli');
		})();
		</script>
	</body>
</html>
