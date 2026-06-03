#[test]
fn php_install_packages_include_matching_fpm_package() {
    for version in devpanel::core::app_config::php_version_numbers() {
        let packages = devpanel::operations::php::php_packages_for_version(&version, true);

        assert!(packages.contains(&format!("php{version}")));
        assert!(packages.contains(&format!("php{version}-fpm")));
    }
}

#[test]
fn php_remove_packages_include_matching_fpm_package() {
    let packages = devpanel::operations::php::php_packages_for_version("8.2", false);

    assert!(packages.contains(&"php8.2".to_string()));
    assert!(packages.contains(&"php8.2-fpm".to_string()));
    assert!(packages.contains(&"php8.2-*".to_string()));
}

#[tokio::test]
async fn dry_run_php_install_preview_includes_fpm_package() {
    let output = devpanel::operations::php::apt_php_op("", "8.2", true)
        .await
        .expect("dry-run PHP install should return a command preview");

    assert!(output.contains("apt-get -y install"));
    assert!(output.contains("php8.2"));
    assert!(output.contains("php8.2-fpm"));
}

#[tokio::test]
async fn dry_run_ondrej_php_ppa_check_is_safe_preview() {
    let output = devpanel::operations::php::ensure_ondrej_php_ppa("")
        .await
        .expect("dry-run PPA setup should return a preview");

    assert_eq!(output, "Would ensure ppa:ondrej/php is configured");
}

#[tokio::test]
async fn dry_run_php_switch_switches_cli_and_fpm() {
    let output = devpanel::operations::php::switch_php("", "8.2")
        .await
        .expect("dry-run PHP switch should succeed");

    assert_eq!(output, "PHP 8.2 selected for CLI and PHP-FPM");
}
