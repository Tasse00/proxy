## Proxy

简易代理工具

### 用途举例

1. mysql代理

    当线上mysql默认监听localhost且临时存在局域网访问的需求时,启动一个监听任意地址并转发到localhost的代理

    ``` bash
    $ RUST_LOG=info ./proxy --listen=0.0.0.0:3307 --target=localhost:3306
    Jun 21 13:51:10.033  INFO proxy: listen="0.0.0.0:3307" target="localhost:3306" buffer_size=4096
    ```

2. ...