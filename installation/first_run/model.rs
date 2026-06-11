#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstRunPackage {
    ProjectsDir,
    Apache,
    Php,
    Mysql,
    PhpExtras,
    Composer,
    NodeNvm,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FirstRunInstallOptions {
    pub install_apache: bool,
    pub install_php: bool,
    pub install_mysql: bool,
    pub install_php_extras: bool,
    pub install_composer: bool,
    pub install_node_nvm: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstRunPackageStatus {
    Installed,
    NotInstalled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstRunSetupStatus {
    pub projects_dir: FirstRunPackageStatus,
    pub apache: FirstRunPackageStatus,
    pub php: FirstRunPackageStatus,
    pub mysql: FirstRunPackageStatus,
    pub php_extras: FirstRunPackageStatus,
    pub composer: FirstRunPackageStatus,
    pub node_nvm: FirstRunPackageStatus,
}

impl Default for FirstRunSetupStatus {
    fn default() -> Self {
        Self {
            projects_dir: FirstRunPackageStatus::NotInstalled,
            apache: FirstRunPackageStatus::NotInstalled,
            php: FirstRunPackageStatus::NotInstalled,
            mysql: FirstRunPackageStatus::NotInstalled,
            php_extras: FirstRunPackageStatus::NotInstalled,
            composer: FirstRunPackageStatus::NotInstalled,
            node_nvm: FirstRunPackageStatus::NotInstalled,
        }
    }
}

impl FirstRunSetupStatus {
    pub fn status_for(&self, package: FirstRunPackage) -> FirstRunPackageStatus {
        match package {
            FirstRunPackage::ProjectsDir => self.projects_dir,
            FirstRunPackage::Apache => self.apache,
            FirstRunPackage::Php => self.php,
            FirstRunPackage::Mysql => self.mysql,
            FirstRunPackage::PhpExtras => self.php_extras,
            FirstRunPackage::Composer => self.composer,
            FirstRunPackage::NodeNvm => self.node_nvm,
        }
    }
}
