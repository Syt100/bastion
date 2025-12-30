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
  messages: {
    initializedPleaseSignIn: 'Initialized. Please sign in.',
  },
  errors: {
    loginFailed: 'Login failed',
    logoutFailed: 'Logout failed',
    setupFailed: 'Setup failed',
    passwordsDoNotMatch: 'Passwords do not match',
  },
} as const

export default messages

