import { EndedGameState } from "./model";

const GameResults: React.FC<{
  ws: WebSocket;
  state: EndedGameState;
}> = ({ ws, state }) => {
  return (
    <div>
      <h2>Game Results</h2>
      <ul>
        {state.users.map((user) => (
          <li key={user.name}>
            {user.name}: {user.score}
          </li>
        ))}
      </ul>
      <button
        onClick={() => {
          ws.send(JSON.stringify({ type: "NewGame" }));
        }}
      >
        New Game
      </button>
    </div>
  );
};

export default GameResults;
