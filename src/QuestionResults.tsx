import { Flex, Heading, List, Text } from "@chakra-ui/react";
import { WaitingForNextQuestionState } from "./model";
import Scoreboard from "./components/Scoreboard";

const QuestionResults: React.FC<{ state: WaitingForNextQuestionState }> = ({
  state,
}) => {
  state.correct_submissions.sort(
    (a, b) => a.submitted_at_ms - b.submitted_at_ms,
  );
  return (
    <Flex direction="column" gap="2">
      <div>
        <Text textStyle="xl">
          Correct answer is&nbsp;
          <Text textStyle="xl" color="green.500" as="span">
            {state.answer.name}!
          </Text>
        </Text>
      </div>
      {state.correct_submissions.length > 0 && (
        <Flex direction="column" gap="2">
          <Heading size="xl">Results</Heading>
          <List.Root>
            {state.correct_submissions.map((sub, i) => {
              return (
                <List.Item key={i}>
                  {sub.user_name}: {sub.submitted_at_ms / 1000}s
                </List.Item>
              );
            })}
          </List.Root>
        </Flex>
      )}
      <Scoreboard
        title="Scoreboard"
        users={state.users}
        submissions={state.correct_submissions}
      />
    </Flex>
  );
};

export default QuestionResults;
