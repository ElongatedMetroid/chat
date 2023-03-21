# chat
Tcp chat made in Rust with a GUI

## TODO
Clean up connection code, and in general clean up client_streams module
Create guidelines for messages and username (data that is sent in general), also guidlines for max amount of clients
Enforce options in MessageGuideline config
Clean up all code
Allow errors to be handled (Currently client is disconnected, maybe wait for a response from the client for like 2 seconds before dissconnecting)