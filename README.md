# Vortex

**Vortex** is a multiplayer platform for turn-based games such as Chess, Scribble, Monopoly, and more.  
> **Note:** Currently, only **Chess** is supported. Additional games will be added in future updates.

## 🧩 Features

### 👥 Multiplayer Gameplay
- Play turn-based games with friends or matched players.
- Friends on your list can **spectate** your games, enhancing community engagement.

### 🎮 Game Types

- **Staked Games**  
  - Players place a bet on their victory.  
  - Spectators can also place bets within the first 5 minutes of the match.

- **Normal Games**  
  - Function exactly like staked games, but **without any monetary bets**.

### 🕹 Game Modes

- **Create a Lobby**  
  - Start a private lobby and invite friends to play.
  - Currently supports **2 players** (since only Chess is available).

- **Find a Match**  
  - Automatically match with random players.
  - **Staked players** are only matched with other staked players.

### 🔁 Replay Feature

- Rewatch completed matches at any time.

  - **Normal Games:** Restart automatically.
  - **Staked Games:** Require players to place new bets to restart.

> **Note:** Every match (including replays) is treated as a unique, separate entity.  
> Bets are evaluated independently for each instance.


## Project Demo
[Vortex Project Demo](https://drive.google.com/file/d/1lKqdKbO27KRdyTNZOglrE2yBy8Z1vdj7/view?usp=sharing)

## Project Architecture
[Architecture Design (Lucid Chart)](https://lucid.app/lucidchart/7da583bc-493c-45dc-80b7-34f6002b7646/edit?viewport_loc=-6565%2C-2146%2C8975%2C4355%2C0_0&invitationId=inv_0f90b33d-902f-4d79-b65c-6f4ab7641f46)


## Repo Links

| Codebase              |      Description          |
| :-------------------- | :-----------------------: |
| [Vortex](https://github.com/kratos2377/vortex)    |    Contains Axum APIs for Auth and other services for necessary processing |
| [Vortex-Client](https://github.com/kratos2377/vortex-client)    |  Tauri Client Used to Play/Join Games as Players or specate any games          |
| [Vortex-Mobile](https://github.com/kratos2377/vortex-mobile)            |      React Native App to scan QR codes and stake in the game and check status of any previous bets       |
| [Vortex-Pub-Sub](https://github.com/kratos2377/vortex-pub-sub)|  Elixir Service to broadcast realtime events to players and spectators    |
| [Vortex-Exchange](https://github.com/kratos2377/vortex-exchange)        |  Smart Contracts made using Anchor framework so that players/spectators can place their bets |
| [Executor-Bots](https://github.com/kratos2377/executor-bots)        |  Bots which consume game result events and settle bets for the players |
| [Vortex-Matchmaker](https://github.com/kratos2377/vortex-matchmaker) | Matchmaking Service which matches any two players with similar ratings |



## This Repo Structure

| Codebase              |      Description          |
| :-------------------- | :-----------------------: |
| [messier](crates/messier)    |    Axum API |
| [cerotis](crates/cerotis)    |  Consume Specific Kafka Events and process           |
| [ton](crates/ton)            |      Entity Models        |
| [migration](crates/migration)|       Migration Logic     |
| [orion](crates/orion)        |   Common Models for mongodb , WS and some kafka events |
| [nebula](crates/nebula)        |  API to Stake In Games and find status of those stakes + APIs for DEX as well |
| [nova](crates/nova)        |  Redis Key Subscriber and Game bet Event Publisher |




## Backend Explaination and Documentation
[Backend Explanantion](https://drive.google.com/file/d/12--pbH0VtOc9j9xJzDhI6pODaHniNMmv/view?usp=sharing) | [Backend Documentation](https://docs.google.com/document/d/107DOYn_nzcd1q9lS5SYgMizIAphoP1elpcuMIHjetso/edit?tab=t.0#heading=h.6n58tp4y15mj)

[Messier Documentation](https://docs.google.com/document/d/18qsjmrNxDxH6bXJmC0LRoN6TapAT5TKstKpebk9RJE4/edit?usp=sharing)

[Certois Documentation](https://docs.google.com/document/d/1FKJs6ZrbjGZyUOv30M5G_EwZ4nXzrqB6tUREheL1tVw/edit?usp=sharing)

[Nebula Documentation](https://docs.google.com/document/d/1jqrr5gwWIG-8YOj7Two6TrrLs6TzQGxokGHg3CGkh1g/edit?usp=sharing)

[Nova Documentation](https://docs.google.com/document/d/1jqrr5gwWIG-8YOj7Two6TrrLs6TzQGxokGHg3CGkh1g/edit?usp=sharing)

[Vortex-Pub-Sub Documentation](https://docs.google.com/document/d/13zhCaCjqs_i13Ss73w9zLA13vV-fW3iJUVyxdiPnSNo/edit?usp=sharing)

[Vortex Matchmaker Documentation](https://docs.google.com/document/d/1BG8z9ce1_E4ehz0vYye7lRlKPLs5I8x2yd202_3dZZI/edit?usp=sharing)

[Executor-Bots Documentation](https://docs.google.com/document/d/14tko74CrxQazaVkszIIJTOo1YrVLAI0EyzBW2aXg9gg/edit?usp=sharing)
