import { WaitingGameState } from "./model";
import React from "react";
import Search from "./Search";

const WaitingRoom: React.FC<{ state: WaitingGameState }> = ({ state }) => {
  return (
    <div>
      <h2>Users</h2>
      <ul>
        {state.users.map((user) => (
          <li key={user.name}>{user.name}</li>
        ))}
      </ul>
      <Search />
    </div>
  );
};

export default WaitingRoom;
