<?php require '../config.php'; ?>
<?php loadClass('Helper')->authPage(); ?>
<!DOCTYPE html>
<html lang="en">
	<head>
		<?php echo loadClass('Html')->getHead(true); ?>
	</head>

	<body>
		<?php echo loadClass('Html')->getNavbar(); ?>

		<div class="container">

			<h1>Support</h1>
			<br/>
			<p>I am currently the sole creator and maintainer of the Devilbox and all of its required projects.<br/>If you find this useful or if it makes your daily programming life easier, consider donating to keep this project alive.</p>
			<ul>
				<li><a target="_blank" href="https://github.com/sponsors/cytopia"><strong>GitHub sponsorship</strong></a></li>
				<li><a target="_blank" href="https://opencollective.com/devilbox"><strong>Open Collective</strong></a></li>
				<li><a target="_blank" href="https://www.patreon.com/devilbox"><strong>Patreon</strong></a></li>
			</ul>
			<br/>

			<div class="row">

				<div class="col-md-6">
					<h2>Core projects</h2>
					<p>The following core projects were created and are maintained in order to make the Devilbox work.</p>
					<table class="table table-striped ">
						<thead class="thead-inverse ">
							<tr>
								<th>Repository</th>
								<th>Description</th>
							</tr>
						</thead>
						<tbody>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/Drmzindec/Devilbox-Boost">Devilbox-Boost</a></td>
								<td>Devilbox Boost (this project). PHP 8.3 / 8.4 / 8.5 images are built locally from <code>docker-images/</code></td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/cytopia/devilbox">devilbox</a></td>
								<td>The original Devilbox (upstream, unmaintained since 2023)</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/docker-apache-2.4">docker-apache-2.4</a></td>
								<td>Apache 2.4 images (<code>amd64</code> and <code>arm64</code>)</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/docker-nginx-stable">docker-nginx-stable</a></td>
								<td>Nginx stable images (<code>amd64</code> and <code>arm64</code>)</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/docker-nginx-mainline">docker-nginx-mainline</a></td>
								<td>Nginx mainline images (<code>amd64</code> and <code>arm64</code>)</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/docker-haproxy">docker-haproxy</a></td>
								<td>HA Proxy image</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/docker-ngrok">docker-ngrok</a></td>
								<td>Ngrok image (<code>amd64</code> and <code>arm64</code>)</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/docker-varnish">docker-varnish</a></td>
								<td>Varnish image</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/docker-python-flask">docker-python-flask</a></td>
								<td>Python Flask image</td>
							</tr>
						</tbody>
					</table>
				</div>


				<div class="col-md-6">
					<h2>Supporting projects</h2>
					<p>The following supporting projects were created and are maintained in order to keep the Devilbox eco system running.</p>
					<table class="table table-striped ">
						<thead class="thead-inverse ">
							<tr>
								<th>Repository</th>
								<th>Description</th>
							</tr>
						</thead>
						<tbody>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/cert-gen">cert-gen</a></td>
								<td>CA and cert generation tool</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/vhost-gen">vhost-gen</a></td>
								<td>HTTPD agnostic vhost creation tool</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/watcherd">watcherd</a></td>
								<td>OS agnostic filesystem change poller tool</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/makefiles">makefiles</a></td>
								<td>Unified build Makefiles</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/github-actions">github-actions</a></td>
								<td>Re-usable GitHub Action Workflows</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/cytopia/shell-command-retry-action">shell-command-retry</a></td>
								<td>GitHub Action</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/cytopia/docker-tag-action">docker-tag</a></td>
								<td>GitHub Action</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/cytopia/git-ref-matrix-action">git-ref-matrix</a></td>
								<td>GitHub Action</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/cytopia/upload-artifact-verify-action">upload-artifact-verify</a></td>
								<td>GitHub Action</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/docker-python-sphinx">docker-python-sphinx</a></td>
								<td>Dockerized Sphinx documentation builder</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/xdebug">xdebug</a></td>
								<td>IP addr alias MacOS plist for Xdebug</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/cytopia/docker-ansible">docker-ansible</a></td>
								<td>Ansible images for PHP-FPM Dockerfile creation</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/cytopia/linkcheck">linkcheck</a></td>
								<td>Broken linkchecker for documentation</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/cytopia/docker-linkcheck">docker-linkcheck</a></td>
								<td>Dockerized version of linkcheck</td>
							</tr>
							<tr>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/devilbox/docker-php-1.99s">docker-php-1.99s</a></td>
								<td>Just for the lulz</td>
							</tr>
						</tbody>
					</table>
				</div>

			</div>

		</div><!-- /.container -->

		<?php echo loadClass('Html')->getFooter(); ?>
	</body>
</html>
