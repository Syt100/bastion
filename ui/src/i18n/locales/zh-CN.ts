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
    refresh: '刷新',
    close: '关闭',
    cancel: '取消',
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
  agents: {
    title: '客户端',
    subtitle: '管理连接到 Hub 的客户端（Agent）。',
    newToken: '创建注册令牌',
    revokeConfirm: '确认撤销该客户端？撤销后需要重新注册才能连接。',
    status: {
      online: '在线',
      offline: '离线',
      revoked: '已撤销',
    },
    columns: {
      name: '名称',
      id: 'ID',
      status: '状态',
      lastSeen: '最后在线',
      actions: '操作',
    },
    actions: {
      copy: '复制',
      copyToken: '复制令牌',
      revoke: '撤销',
    },
    tokenModal: {
      title: '创建注册令牌',
      ttl: '有效期（秒）',
      remainingUses: '可用次数（可选）',
      create: '创建',
      help: '将令牌粘贴到客户端的注册步骤中完成绑定。',
      token: '令牌',
      expiresAt: '过期时间',
    },
  },
  messages: {
    initializedPleaseSignIn: '初始化完成，请登录。',
    enrollmentTokenCreated: '注册令牌已创建',
    copied: '已复制',
    agentRevoked: '客户端已撤销',
  },
  errors: {
    loginFailed: '登录失败',
    logoutFailed: '退出失败',
    setupFailed: '初始化失败',
    passwordsDoNotMatch: '两次输入的密码不一致',
    fetchAgentsFailed: '获取客户端列表失败',
    createEnrollmentTokenFailed: '创建注册令牌失败',
    revokeAgentFailed: '撤销客户端失败',
    copyFailed: '复制失败',
  },
} as const

export default messages
