import { useEffect, useMemo, useState } from "react";
import { PlayingGameState, Question, User, UserGameState } from "./model.tsx";
import { Button, Heading, List, Progress } from "@chakra-ui/react";
import { post } from "./utils.tsx";

const Game: React.FC<{
  ws: WebSocket;
  state: PlayingGameState;
  user: User;
  room: string;
}> = ({ ws, state, user, room }) => {
  const [users, setUsers] = useState<Array<UserGameState>>([]);
  const [questionId, setQuestionId] = useState<number>(-1);
  const [question, setQuestion] = useState<Question | null>(null);
  const [selectedChoice, setSelectedChoice] = useState<number | null>(null);
  const [audioCurrentTime, setAudioCurrentTime] = useState<number>(0);
  const audio = useMemo(() => new Audio(), []);

  useEffect(() => {
    if (state.question_id !== questionId) {
      setQuestionId(state.question_id);
      setQuestion(state.question);
      setSelectedChoice(null);
      audio.src = state.question.song_url;
      audio.currentTime = state.song_progress_ms / 1000;
    }

    if (state.users !== users) {
      setUsers(state.users);
    }
  }, [state, questionId, users, audio]);

  useEffect(() => {
    audio.autoplay = true;
    audio.volume = 0.5;
    audio.ontimeupdate = () => {
      setAudioCurrentTime(audio.currentTime);
    };
    return () => {
      audio.pause();
      audio.src = "";
    };
  }, [audio]);

  const handleChoiceSubmit = (selectedChoice: number) => {
    setSelectedChoice(selectedChoice);
    ws.send(
      JSON.stringify({
        type: "UserSubmitted",
        user_id: user.id,
        choice: selectedChoice,
      }),
    );
  };

  if (question === null) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <Heading size="4xl">Question {questionId + 1}</Heading>
      <Progress.Root
        value={(audioCurrentTime / 10) * 100}
        colorPalette="green"
        width="xl"
      >
        <Progress.Track>
          <Progress.Range />
        </Progress.Track>
      </Progress.Root>
      {question.choices.map((choice, index) => (
        <Button
          key={index}
          type="button"
          onClick={() => handleChoiceSubmit(index)}
          disabled={selectedChoice !== null}
          style={{
            backgroundColor: selectedChoice === index ? "blue" : "gray",
            color: "white",
            margin: "5px",
            padding: "10px",
          }}
        >
          {choice.name}
        </Button>
      ))}
      <Heading size="4xl">Scoreboard</Heading>
      <List.Root>
        {users.map((user) => (
          <List.Item key={user.name}>
            {user.name}: {user.score}
          </List.Item>
        ))}
      </List.Root>
      <Button
        onClick={() => {
          post(`/api/room/${room}/reset`, {});
        }}
      >
        Back to waiting room
      </Button>
    </div>
  );
};

export default Game;
