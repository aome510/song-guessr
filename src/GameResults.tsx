import { EndedGameState } from "./model";
import { post } from "./utils";

const GameResults: React.FC<{
  room: string;
  state: EndedGameState;
}> = ({ room, state }) => {
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
          post(`/api/room/${room}/reset`, {});
        }}
      >
        New Game
      </button>
    </div>
  );
};

export default GameResults;
