# 通讯协议

都为小端

## 客户端通信

    要做的功能:
    1. 握手
    2. 维持心跳
    3. 客户端订阅消息
    4. 服务器推送信息
    5. 客户端推送消息
    6. 提供应答
    7. 推拉状态转换

协议类型:

    服务器信息 => 0
    客户端信息 => 1
    ping => 2
    pong => 3
    信息 => 4
    请求信息 => 5
    应答 => 6
    订阅 => 7
    取消订阅 => 8
    错误 => 9
    转为推 => 10
    转为拉 => 11

1. 握手

首先, 由服务器提供服务信息

    |1字节|1字节|2字节|4字节|
    |类型|版本|支持的服务的位掩码|可接收内容的最大长度|

位掩码:
    
    推消息 => 1
    拉消息 => 2
    支持tls => 4
    支持压缩 => 8


其次, 由客户端提供信息

    |1字节|1字节|2字节|1字节|
    |类型|版本|支持的服务的位掩码|客户端可容纳的消息数量|