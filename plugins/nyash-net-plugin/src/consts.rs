// Extracted constants for nyash-net-plugin

// Error codes
pub(crate) const OK: i32 = 0;
pub(crate) const E_SHORT: i32 = -1;
pub(crate) const _E_INV_TYPE: i32 = -2;
pub(crate) const E_INV_METHOD: i32 = -3;
pub(crate) const E_INV_ARGS: i32 = -4;
pub(crate) const E_ERR: i32 = -5;
pub(crate) const E_INV_HANDLE: i32 = -8;

// Type IDs
pub(crate) const _T_SERVER: u32 = 20;
pub(crate) const T_REQUEST: u32 = 21;
pub(crate) const T_RESPONSE: u32 = 22;
pub(crate) const _T_CLIENT: u32 = 23;
// Socket
pub(crate) const _T_SOCK_SERVER: u32 = 30;
pub(crate) const T_SOCK_CONN: u32 = 31;
pub(crate) const _T_SOCK_CLIENT: u32 = 32;

// Methods
pub(crate) const M_BIRTH: u32 = 0;

// Server
pub(crate) const M_SERVER_START: u32 = 1;
pub(crate) const M_SERVER_STOP: u32 = 2;
pub(crate) const M_SERVER_ACCEPT: u32 = 3; // -> Handle(Request)

// Request
pub(crate) const M_REQ_PATH: u32 = 1; // -> String
pub(crate) const M_REQ_READ_BODY: u32 = 2; // -> Bytes (optional)
pub(crate) const M_REQ_RESPOND: u32 = 3; // arg: Handle(Response)

// Response
pub(crate) const M_RESP_SET_STATUS: u32 = 1; // arg: i32
pub(crate) const M_RESP_SET_HEADER: u32 = 2; // args: name, value (string)
pub(crate) const M_RESP_WRITE: u32 = 3; // arg: bytes/string
pub(crate) const M_RESP_READ_BODY: u32 = 4; // -> Bytes
pub(crate) const M_RESP_GET_STATUS: u32 = 5; // -> i32
pub(crate) const M_RESP_GET_HEADER: u32 = 6; // arg: name -> string (or empty)

// Client
pub(crate) const M_CLIENT_GET: u32 = 1; // arg: url -> Handle(Response)
pub(crate) const M_CLIENT_POST: u32 = 2; // args: url, body(bytes/string) -> Handle(Response)

// Socket Server
pub(crate) const M_SRV_BIRTH: u32 = 0;
pub(crate) const M_SRV_START: u32 = 1; // port
pub(crate) const M_SRV_STOP: u32 = 2;
pub(crate) const M_SRV_ACCEPT: u32 = 3; // -> Handle(T_SOCK_CONN)
pub(crate) const M_SRV_ACCEPT_TIMEOUT: u32 = 4; // ms -> Handle(T_SOCK_CONN) or void

// Socket Client
pub(crate) const M_SC_BIRTH: u32 = 0;
pub(crate) const M_SC_CONNECT: u32 = 1; // host, port -> Handle(T_SOCK_CONN)

// Socket Conn
pub(crate) const M_CONN_BIRTH: u32 = 0;
pub(crate) const M_CONN_SEND: u32 = 1; // bytes/string -> void
pub(crate) const M_CONN_RECV: u32 = 2; // -> bytes
pub(crate) const M_CONN_CLOSE: u32 = 3; // -> void
pub(crate) const M_CONN_RECV_TIMEOUT: u32 = 4; // ms -> bytes (empty if timeout)
