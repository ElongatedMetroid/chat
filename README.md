# chat
Tcp chat made in Rust with a GUI

## TODO
Clean up connection code, and in general clean up client_streams module
Create guidelines for messages and username (data that is sent in general), also guidlines for max amount of clients
Enforce options in MessageGuideline config
Clean up all code
Remove Username struct and just update UserBuilder to have a with_guidelines method
On a client connect the server will send its guidelines, this means the server dropping anything that does not follow these will not be as bad of a practice.