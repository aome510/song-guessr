import { Flex, Heading, Table, Text } from "@chakra-ui/react";
import { WaitingForNextQuestionState } from "./model";
import Scoreboard from "./components/Scoreboard";

const QuestionResults: React.FC<{ state: WaitingForNextQuestionState }> = ({
  state,
}) => {
  state.submissions.sort((a, b) => a.submitted_at_ms - b.submitted_at_ms);
  return (
    <Flex direction="column" gap="2">
      <div>
        <Text textStyle="lg">
          Correct answer is&nbsp;
          <Text textStyle="lg" color="green.500" as="span">
            {state.choices[state.answer_id]}!
          </Text>
        </Text>
      </div>

      {state.submissions.length > 0 && (
        <Flex direction="column" gap="2">
          <Heading size="xl">Submissions</Heading>
          <Table.Root variant="outline" size="md">
            <Table.Header>
              <Table.Row>
                <Table.ColumnHeader>User</Table.ColumnHeader>
                <Table.ColumnHeader>Selected Answer</Table.ColumnHeader>
                <Table.ColumnHeader>Time</Table.ColumnHeader>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {state.submissions.map((sub, i) => {
                const isCorrect = sub.selected_id === state.answer_id;
                return (
                  <Table.Row key={i}>
                    <Table.Cell>{sub.user_name}</Table.Cell>
                    <Table.Cell>
                      <Text
                        as="span"
                        color={isCorrect ? "green.500" : "red.500"}
                        fontWeight="semibold"
                      >
                        {isCorrect ? "✓" : "✗"}{" "}
                      </Text>
                      {state.choices[sub.selected_id]}
                    </Table.Cell>
                    <Table.Cell>
                      {(sub.submitted_at_ms / 1000).toFixed(2)}s
                    </Table.Cell>
                  </Table.Row>
                );
              })}
            </Table.Body>
          </Table.Root>
        </Flex>
      )}

      <Scoreboard
        title="Scoreboard"
        users={state.users}
        submissions={state.submissions}
      />
    </Flex>
  );
};

export default QuestionResults;
