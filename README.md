# Vortex


Multiplayer Backend Server for my games like chess and Monopoly , (Games which can run on TCP)


To use this its good to send something like game event where each event will tell us which user made the move and which room should we broadcast it to and then we can parse event according to the game which is in the event

## Todo
- [ ] Refactor code
- [ ] Add Kafka Schemas in helix



## Features to add
- [ ] Users can stake money in games
- [ ] Few games like chess and poker can be simple games or staked game
- [ ] Users can manage their sols by investing in DEXes

## Structure

| Codebase              |      Description          |
| :-------------------- | :-----------------------: |
| [messier](crates/messier)    |    Axum API and WS Events |
| [cerotis](crates/cerotis)    |  Consume Kafka Events and Send Events to MQTT Broker          |
| [ton](crates/ton)            |      Entity Models        |
| [migration](crates/migration)|       Migration Logic     |
| [saggitarius](crates/saggitarius)        |   Mongo Kafka Streams Consumer        |
| [helix](crates/helix)        |   Kafka Schema Publisher |
| [orion](crates/orion)        |   Common Models for mongodb , WS and some kafka events |