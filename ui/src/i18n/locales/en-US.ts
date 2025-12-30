const messages = {
  app: {
    name: 'Bastion',
  },
  common: {
    mvp: 'MVP',
    light: 'Light',
    dark: 'Dark',
    logout: 'Logout',
    language: 'Language',
    refresh: 'Refresh',
    close: 'Close',
    cancel: 'Cancel',
  },
  nav: {
    dashboard: 'Dashboard',
    jobs: 'Jobs',
    agents: 'Agents',
    settings: 'Settings',
  },
  dashboard: {
    title: 'Dashboard',
    subtitle: 'Backup activity overview (placeholder data).',
    runs7d: 'Runs (Last 7 Days)',
  },
  placeholder: {
    comingSoon: 'Coming soon.',
  },
  auth: {
    signIn: 'Sign in',
    username: 'Username',
    password: 'Password',
    login: 'Login',
    initTitle: 'Initialize Bastion',
    confirmPassword: 'Confirm password',
    initialize: 'Initialize',
  },
  agents: {
    title: 'Agents',
    subtitle: 'Manage agents connected to the Hub.',
    newToken: 'Create enrollment token',
    revokeConfirm: 'Revoke this agent? It must re-enroll to connect again.',
    status: {
      online: 'Online',
      offline: 'Offline',
      revoked: 'Revoked',
    },
    columns: {
      name: 'Name',
      id: 'ID',
      status: 'Status',
      lastSeen: 'Last seen',
      actions: 'Actions',
    },
    actions: {
      copy: 'Copy',
      copyToken: 'Copy token',
      revoke: 'Revoke',
    },
    tokenModal: {
      title: 'Create enrollment token',
      ttl: 'TTL (seconds)',
      remainingUses: 'Remaining uses (optional)',
      create: 'Create',
      help: 'Paste the token into the agent enrollment step to bind it.',
      token: 'Token',
      expiresAt: 'Expires at',
    },
  },
  messages: {
    initializedPleaseSignIn: 'Initialized. Please sign in.',
    enrollmentTokenCreated: 'Enrollment token created',
    copied: 'Copied',
    agentRevoked: 'Agent revoked',
  },
  errors: {
    loginFailed: 'Login failed',
    logoutFailed: 'Logout failed',
    setupFailed: 'Setup failed',
    passwordsDoNotMatch: 'Passwords do not match',
    fetchAgentsFailed: 'Failed to fetch agents',
    createEnrollmentTokenFailed: 'Failed to create enrollment token',
    revokeAgentFailed: 'Failed to revoke agent',
    copyFailed: 'Copy failed',
  },
} as const

export default messages
