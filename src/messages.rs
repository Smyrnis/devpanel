use iced::widget::text_editor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Repos,
    VHosts,
    SshKeys,
    Tools,
    Config,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectTab(Tab),
    NotificationTick,
    DismissAllNotifications,
    Dashboard(DashboardMessage),
    SshKeys(SshKeysMessage),
    Tools(ToolsMessage),
    Repos(ReposMessage),
    VHosts(VHostsMessage),
    Sudo(SudoMessage),
    FirstRun(FirstRunMessage),
    Config(ConfigMessage),
}

#[derive(Debug, Clone)]
pub enum DashboardMessage {
    RefreshStatus,
    StatusRefreshed {
        apache: bool,
        mysql: bool,
        php: Option<String>,
        php_versions: Vec<String>,
        apache_uptime: Option<String>,
        mysql_uptime: Option<String>,
        recent_failures: Vec<String>,
    },
    AutoRefreshTick,
    #[allow(dead_code)]
    ResetIssuesCheck,
    StartApache,
    StopApache,
    RestartApache,
    StartMySQL,
    StopMySQL,
    RestartMySQL,
    RestartAll,
    ServiceResult {
        service: String,
        action: String,
        success: bool,
        output: String,
    },
    SwitchPhpVersion(String),
    PhpSwitchResult(bool, String),
    ShowPhpInfo,
    PhpInfoLoaded(String),
    ClosePhpInfo,
    OpenLocalhost,
    OpenPhpMyAdmin,
    OpenWebRoot,
    OpenProjectsFolder,
    NavigateApache2Conf,
    NavigateApache2Sites,
    NavigatePhpDir,
    NavigateMysqlDir,
    NavigateHostsFile,
    OpenPhpIni,
}

#[derive(Debug, Clone)]
pub enum SshKeysMessage {
    EmailChanged(String),
    KeyNameChanged(String),
    KeyTypeChanged(crate::tabs::ssh_keys::KeyType),
    PassphraseChanged(String),
    TogglePassphrase(bool),
    GenerateKey,
    GenerateDone(bool, String),
    AddExisting,
    AddExistingDone(bool, String),
    OpenDir,
    ListKeys,
    KeysListed(Vec<crate::tabs::ssh_keys::KeyEntry>),
    CopyPublicKey(String),
    CopyPublicKeyDone(bool, String),
}

#[derive(Debug, Clone)]
pub enum ToolsMessage {
    ScanPhp,
    ScanDone(Vec<(String, crate::tabs::tools::PhpStatus, bool, bool, bool)>),
    InstallPhp(String),
    RemovePhp(String),
    PhpOpDone(bool, String),
    OpenMysqlCli,
    OpenMariadbCli,
    OpenMysqlSocket,
    ClearLog,
    CopyFixCommands(String),
    CopyDone,
    SetSection(crate::tabs::tools::ToolSection),
    ToolSearchChanged(String),
    ScanInstalledTools,
    InstalledToolsScanned(crate::tabs::tools::InstalledTools),
    InstallComposer,
    UpdateComposer,
    ComposerDone(bool, String),
    CopyNvmInstallCommand,
    RedisStart,
    RedisStop,
    RedisDone(bool, String),
    ScanApacheMods,
    ScanApacheModsDone(Vec<crate::tabs::tools::ApacheModule>),
    ModFilterChanged(String),
    EnableApacheMod(String),
    DisableApacheMod(String),
    ApacheModDone(bool, String, String, bool),
    ScanPhpExts,
    ScanPhpExtsDone(Vec<(String, bool)>),
    InstallPhpExt(String),
    RemovePhpExt(String),
    PhpExtDone(bool, String),
}

#[derive(Debug, Clone)]
pub enum ReposMessage {
    CheckSsh,
    SshChecked(bool, String, bool, String),
    Fetch,
    FetchDone(Vec<crate::tabs::repos::RemoteRepo>),
    NextPage,
    PrevPage,
    Clone { ssh_url: String, name: String },
    CloneDone(bool, String, String),
    OpenCloned(String),
    OpenEditor(String),
    SearchChanged(String),
    SetFilter(crate::tabs::repos::ProviderFilter),
    OpenRoot,
}

#[derive(Debug, Clone)]
pub enum VHostsMessage {
    Scan,
    ScanDone(Vec<crate::tabs::vhosts::VHostEntry>),
    ShowAddForm,
    HideForm,
    FormServerNameChanged(String),
    FormDocRootChanged(String),
    FormPhpVersionChanged(Option<String>),
    FormHttpsChanged(bool),
    Create,
    CreateDone(bool, String),
    EditRequest(usize),
    SaveEdit,
    SaveEditDone(bool, String),
    OpenBrowser(String),
    OpenDevpanelConf,
    DeleteRequest(usize),
    DeleteConfirm(usize),
    BulkDeleteConfirm,
    DeleteCancel,
    DeleteDone(bool, String),
    ToggleSelected(usize),
    SelectAll,
    ClearSelection,
    BulkTagChanged(String),
    ApplyBulkTag,
    ToggleHttps(usize),
    DuplicateRequest(usize),
    OpenConfigEditor,
    CloseConfigEditor,
    ConfigLoaded(String),
    ConfigEditorAction(text_editor::Action),
    SaveConfigFile,
    SaveConfigDone(bool, String),
}

#[derive(Debug, Clone)]
pub enum SudoMessage {
    PasswordChanged(String),
    ToggleShow(bool),
    ToggleSave(bool),
    Cancel,
    Submit,
    ValidationResult(bool),
    ClearSaved,
}

#[derive(Debug, Clone)]
pub enum FirstRunMessage {
    Continue,
    Exit,
    ToggleMysql(bool),
    TogglePhpExtras(bool),
    ProgressTick,
    LogLoaded(Vec<String>),
    InstallDone(bool, String),
}

#[derive(Debug, Clone)]
pub enum ConfigMessage {
    SetSection(crate::tabs::config::ConfigSection),
    Save,
    SaveDone(bool, String),
    ApacheLogLevelChanged(String),
    ApacheAutoReloadChanged(bool),
    PhpDefaultVersionChanged(String),
    PhpDisplayErrorsChanged(bool),
    ProjectsOpenCommandChanged(String),
    UiConfirmDeletesChanged(bool),
    UiToastDurationChanged(u32),
    UiShowSetupLogChanged(bool),
    SshDefaultKeyTypeChanged(String),
    EditorCommandChanged(String),
}
