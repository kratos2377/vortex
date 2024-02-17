# Vortex


Multiplayer Backend Server for my games like chess and Monopoly , (Games which can run on TCP)


To use this its good to send something like game event where each event will tell us which user made the move and which room should we broadcast it to and then we can parse event according to the game which is in the event


## Todos
- [ ] User can create lobbies and invite their friends to join
- [ ] WS implementation so that we can play any game
- [ ] Add Either Session Based auth or JWT


## APIS left
- [ ] User Friend Request APIS (CREATE DELETE ACCEPT/REJECT)
- [ ] User can add/delete different wallets
- [ ] Fix User Friends Scehma
