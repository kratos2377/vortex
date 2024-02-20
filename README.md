# Vortex


Multiplayer Backend Server for my games like chess and Monopoly , (Games which can run on TCP)


To use this its good to send something like game event where each event will tell us which user made the move and which room should we broadcast it to and then we can parse event according to the game which is in the event

## Todo
- [x] User Friend Request APIS (CREATE DELETE ACCEPT/REJECT)
- [x] User can add/delete different wallets
- [x] Fix User Friends Scehma
- [ ] Add sea-orm migration
- [ ] Users can see their online friends
- [ ] Add WS events
- [ ] Add kafka/grpc for some realtime streaming for some use cases
