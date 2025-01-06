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
      <Flex direction="column" gap="2">
        <Button
          onClick={() => {
            post(`/api/room/${room}/restart`, {});
          }}
        >
          Restart Game
        </Button>
        <Button
          onClick={() => {
            post(`/api/room/${room}/reset`, {});
          }}
        >
          Back to Lobby
        </Button>
      </Flex>
    </Flex>
  );
};

export default GameResults;
