# wasmesh

wasmesh(WebAssembly Service Mesh) 是基于 WebAssembly 的微服务治理框架.

## Host-Guest 交互设计

- host handler 接收外部请求
- host 回调 guest handler 函数
- guest 处理请求
  - 无嵌套交互，直接返回 Reply/Exception 的 offset+len
  - 有嵌套交互，回调 host client 函数
    - Call：阻塞 收到响应 Reply 的 offset+len
    - Oneway：收到发送结果的 Exception 的 offset+len
- host handler 收到 Reply/Exception 并返回给外部调用者
