import { Flex } from "@chakra-ui/react";
import { WaitingForNextQuestionState } from "./model";
import Scoreboard from "./components/Scoreboard";

const QuestionResults: React.FC<{ state: WaitingForNextQuestionState }> = ({
  state,
}) => {
  return (
    <Flex>
      <Scoreboard title="Scoreboard" users={state.users} />
    </Flex>
  );
};

export default QuestionResults;
