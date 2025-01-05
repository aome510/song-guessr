import { WaitingGameState } from "./model";
import React from "react";
import Search from "./Search";
import { Heading, List } from "@chakra-ui/react";

const WaitingRoom: React.FC<{ state: WaitingGameState; id: string }> = ({
  state,
  id,
}) => {
  return (
    <div>
      <Heading size="4xl">Users</Heading>
      <List.Root>
        {state.users.map((user, i) => (
          <List.Item key={i}>{user.name}</List.Item>
        ))}
      </List.Root>
      <Search room={id} />
    </div>
  );
};

export default WaitingRoom;
