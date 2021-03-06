use std::convert::{Into, TryInto};
use std::ops::{BitAnd, BitOrAssign};

// 服务器信息
pub(crate) const STATE_SERVER_INFO: u8 = 0;

// 客户端信息
pub(crate) const STATE_CLIENT_INFO: u8 = 1;

// 心跳发出
pub(crate) const STATE_PING: u8 = 2;

// 心跳应答
pub(crate) const STATE_PONG: u8 = 3;

// 消息
pub(crate) const STATE_MSG: u8 = 4;

// 消息序号, 用于拉取消息的时候用
pub(crate) const STATE_OFFSET: u8 = 5;

// 应答收到消息
pub(crate) const STATE_ACK: u8 = 6;

// 订阅消息
pub(crate) const STATE_SUB: u8 = 7;

// 发布消息
pub(crate) const STATE_PUB: u8 = 8;

// 取消订阅
pub(crate) const STATE_UNSUB: u8 = 9;

// 错误
pub(crate) const STATE_ERR: u8 = 10;

// 转为推消息
pub(crate) const STATE_TURN_PUSH: u8 = 11;

// 转为拉消息
pub(crate) const STATE_TURN_PULL: u8 = 12;

// 确认, 回答 turn_push 或 turn_pull
pub(crate) const STATE_OK: u8 = 13;

// 服务器解析协议状态
#[derive(Debug)]
pub(super) enum ServerState {
    // 客户端信息
    ClientInfo,

    Ping,
    Pong,

    // 解析接受发布消息的消息号
    Offset,
    Ack,

    // 解析错误
    Err,

    // 解析错误内容
    ErrContent,
    TurnPush,
    TurnPull,
    Ok,

    // 解析发布
    Pub,

    // 解析发布名称的长度
    PubSubNameLength,

    // 解析发布名称, 用于识别订阅名称
    PubSubName,

    // 解析发布内容长度
    PubMsgLength,

    // 解析发布内容
    PubMsg,

    // 订阅
    Sub,

    // 解析订阅名称的长度
    SubNameLength,

    // 解析订阅名称
    SubName,

    // 解析取消订阅
    UnSub,

    // 解析取消订阅的数量
    UnSubTotal,

    // 解析取消订阅名称长度
    UnSubNameLength,

    // 解析取消订阅名称
    UnSubName,
}

impl TryInto<ServerState> for u8 {
    type Error = ();

    fn try_into(self) -> Result<ServerState, Self::Error> {
        match self {
            STATE_CLIENT_INFO => Ok(ServerState::ClientInfo),
            STATE_PING => Ok(ServerState::Ping),
            STATE_PONG => Ok(ServerState::Pong),
            STATE_OFFSET => Ok(ServerState::Offset),
            STATE_ACK => Ok(ServerState::Ack),
            STATE_ERR => Ok(ServerState::Err),
            STATE_TURN_PULL => Ok(ServerState::TurnPull),
            STATE_TURN_PUSH => Ok(ServerState::TurnPush),
            STATE_OK => Ok(ServerState::Ok),
            STATE_SUB => Ok(ServerState::Sub),
            STATE_PUB => Ok(ServerState::Pub),
            STATE_UNSUB => Ok(ServerState::UnSub),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub(super) enum ClientState {
    ServerInfo,
    Ping,
    Pong,
    Msg,
    MsgOffset,
    MsgSubLength,
    MsgSubName,
    MsgLength,
    MsgPayload,
    Offset,
    Ack,
    Err,
    ErrContent,
    TurnPush,
    TurnPull,
    Ok,
}

impl TryInto<ClientState> for u8 {
    type Error = ();

    fn try_into(self) -> Result<ClientState, Self::Error> {
        match self {
            STATE_SERVER_INFO => Ok(ClientState::ServerInfo),
            STATE_PING => Ok(ClientState::Ping),
            STATE_PONG => Ok(ClientState::Pong),
            STATE_MSG => Ok(ClientState::Msg),
            STATE_OFFSET => Ok(ClientState::Offset),
            STATE_ACK => Ok(ClientState::Ack),
            STATE_ERR => Ok(ClientState::Err),
            STATE_TURN_PULL => Ok(ClientState::TurnPull),
            STATE_TURN_PUSH => Ok(ClientState::TurnPush),
            STATE_OK => Ok(ClientState::Ok),
            _ => Err(()),
        }
    }
}

const SUPPORT_PUSH: u16 = 1;
const SUPPORT_PULL: u16 = 2;
const SUPPORT_TLS: u16 = 4;
const SUPPORT_COMPRESS: u16 = 8;

#[repr(u16)]
#[derive(Debug)]
pub enum Support {
    Push = SUPPORT_PUSH,
    Pull = SUPPORT_PULL,
    Tls = SUPPORT_TLS,
    Compress = SUPPORT_COMPRESS,
}

impl BitOrAssign<Support> for u16 {
    fn bitor_assign(&mut self, rhs: Support) {
        match rhs {
            Support::Push => (*self |= SUPPORT_PUSH),
            Support::Pull => *self |= SUPPORT_PULL,
            Support::Tls => *self |= SUPPORT_TLS,
            Support::Compress => *self |= SUPPORT_COMPRESS,
        }
    }
}

impl BitAnd<Support> for u16 {
    type Output = bool;
    fn bitand(self, rhs: Support) -> Self::Output {
        match rhs {
            Support::Push => (self & SUPPORT_PUSH) == SUPPORT_PUSH,
            Support::Pull => (self & SUPPORT_PULL) == SUPPORT_PULL,
            Support::Tls => (self & SUPPORT_TLS) == SUPPORT_TLS,
            Support::Compress => (self & SUPPORT_COMPRESS) == SUPPORT_COMPRESS,
        }
    }
}
