import { Button, Flex } from "@chakra-ui/react";
import { EndedGameState } from "./model";
import { post } from "./utils";
import Scoreboard from "./components/Scoreboard";

const GameResults: React.FC<{
  room: string;
  state: EndedGameState;
}> = ({ room, state }) => {
  return (
    <Flex direction="column" gap="4">
      <Scoreboard title="Results" users={state.users} />
      <Button
        onClick={() => {
          post(`/api/room/${room}/reset`, {});
        }}
      >
        New Game
      </Button>
    </Flex>
  );
};

export default GameResults;
