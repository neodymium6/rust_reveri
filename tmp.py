import rust_reversi


server = rust_reversi.NetworkArenaServer(10)
server.start("localhost", 12345)
