### 和 AI 一起工作的经验

给定 AI 一个更小的范围，能让 AI 的效果更好。

- 例如，如果你在编写一个前端项目，与其让 AI 构建一整个页面，让它构建一个组件的
  效果会更好。
- 可以先让 AI 构建出一个组件后，反复沟通使得组件效果不错后，再让 AI 参考该组件
  来构建剩余的组件。

如果你的需求可以分为正交的多个子需求，你完全可以开多个 AI CLI 来让它们分别开发这
些子需求。能够最大提升你的效率。

如果你在开发后端的业务项目，请：

- 建议参考 “[规格驱动开发][8]”。不过，我个人感觉没有必要保存规格文件，不过，这个
  行动方式值得参考。
  - 在真实开发前，先定义好经过深思熟虑的规格。并写入到文件中。
  - 然后让 Agent 根据规格完成开发。
- 在规格文件中，定义好：
  - API 接口。包括输入 schema、输出 schema。
  - 数据库表格式。
- 注意，规格文件越详细越好。如果规格文件详细到实习生就能写好的地步，那么 agent
  完成的质量也不会差。

[1]: https://git-scm.com/
[2]: https://prek.j178.dev/
[3]: https://docs.gitlab.com/ee/ci/
[4]: https://github.com/openclaw/openclaw/tree/main/.agents/skills
[5]: https://semver.org/
[6]: https://keepachangelog.com/en/1.1.0/
[7]: https://github.com/casey/just
[8]: https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html
