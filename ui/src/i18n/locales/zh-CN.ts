const messages = {
  app: {
    name: 'Bastion',
  },
  common: {
    mvp: 'MVP',
    light: '浅色',
    dark: '深色',
    logout: '退出登录',
    language: '语言',
  },
  nav: {
    dashboard: '概览',
    jobs: '备份任务',
    agents: '客户端',
    settings: '设置',
  },
  dashboard: {
    title: '概览',
    subtitle: '备份活动概览（示例数据）。',
    runs7d: '近 7 天运行情况',
  },
  placeholder: {
    comingSoon: '即将支持。',
  },
  auth: {
    signIn: '登录',
    username: '用户名',
    password: '密码',
    login: '登录',
    initTitle: '初始化 Bastion',
    confirmPassword: '确认密码',
    initialize: '初始化',
  },
  messages: {
    initializedPleaseSignIn: '初始化完成，请登录。',
  },
  errors: {
    loginFailed: '登录失败',
    logoutFailed: '退出失败',
    setupFailed: '初始化失败',
    passwordsDoNotMatch: '两次输入的密码不一致',
  },
} as const

export default messages

