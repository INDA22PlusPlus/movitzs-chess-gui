// tip: https://docs.rs/prost/latest/prost/trait.Message.html - convert the messages to u8 vecs to
// send them over tcp

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2sConnectRequest {
    #[prost(uint64, tag = "1")]
    pub game_id: u64,
    #[prost(bool, tag = "2")]
    pub spectate: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2cConnectAck {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(uint64, optional, tag = "2")]
    pub game_id: ::core::option::Option<u64>,
    #[prost(message, optional, tag = "3")]
    pub starting_position: ::core::option::Option<BoardState>,
    #[prost(bool, optional, tag = "4")]
    pub client_is_white: ::core::option::Option<bool>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Move {
    #[prost(uint32, tag = "1")]
    pub from_square: u32,
    #[prost(uint32, tag = "2")]
    pub to_square: u32,
    #[prost(enumeration = "Piece", optional, tag = "3")]
    pub promotion: ::core::option::Option<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2cMoveAck {
    #[prost(bool, tag = "1")]
    pub legal: bool,
    #[prost(message, optional, tag = "2")]
    pub board_result: ::core::option::Option<BoardState>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct S2cMessage {
    #[prost(oneof = "s2c_message::Msg", tags = "1, 2, 3")]
    pub msg: ::core::option::Option<s2c_message::Msg>,
}
/// Nested message and enum types in `S2CMessage`.
pub mod s2c_message {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Msg {
        #[prost(message, tag = "1")]
        Move(super::Move),
        #[prost(message, tag = "2")]
        ConnectAck(super::S2cConnectAck),
        #[prost(message, tag = "3")]
        MoveAck(super::S2cMoveAck),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct C2sMessage {
    #[prost(oneof = "c2s_message::Msg", tags = "1, 2")]
    pub msg: ::core::option::Option<c2s_message::Msg>,
}
/// Nested message and enum types in `C2SMessage`.
pub mod c2s_message {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Msg {
        #[prost(message, tag = "1")]
        Move(super::Move),
        #[prost(message, tag = "2")]
        ConnectRequest(super::C2sConnectRequest),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BoardState {
    #[prost(string, tag = "1")]
    pub fen_string: ::prost::alloc::string::String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}
impl Piece {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Piece::Pawn => "Pawn",
            Piece::Knight => "Knight",
            Piece::Bishop => "Bishop",
            Piece::Rook => "Rook",
            Piece::Queen => "Queen",
            Piece::King => "King",
        }
    }
}
