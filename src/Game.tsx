import { useEffect, useMemo, useState } from "react";
import { PlayingGameState, Question, User, UserGameState } from "./model.tsx";
import { Button, Flex, Heading, Progress } from "@chakra-ui/react";
import { post } from "./utils.tsx";
import Scoreboard from "./components/Scoreboard.tsx";

const Game: React.FC<{
  ws: WebSocket;
  state: PlayingGameState;
  user: User;
  room: string;
}> = ({ ws, state, user, room }) => {
  const [users, setUsers] = useState<Array<UserGameState>>([]);
  // doesn't set to be state.question_id to trigger the update defined in the later useEffect
  const [questionId, setQuestionId] = useState<number>(-1);
  const [question, setQuestion] = useState<Question>(state.question);
  const [selectedChoice, setSelectedChoice] = useState<number | null>(null);
  const [audioCurrentTime, setAudioCurrentTime] = useState<number>(0);
  const [audioPlayable, setAudioPlayable] = useState<boolean>(true);
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
        user_name: user.name,
        user_id: user.id,
        choice: selectedChoice,
        submitted_at_ms: Math.round(audio.currentTime * 1000),
      }),
    );
  };

  useEffect(() => {
    const checkAutoPlayable = async () => {
      if (questionId != -1) {
        try {
          if (audio.paused) {
            await audio.play();
          }
          setAudioPlayable(true);
        } catch {
          setAudioPlayable(false);
        }
      }
    };

    checkAutoPlayable();
  }, [questionId, audio]);

  // this is a hack to get the audio to play on the first render
  // because the audio autoplay must be triggered by a user gesture
  // more details: see https://developer.chrome.com/blog/autoplay/
  if (!audioPlayable) {
    return (
      <Button
        padding="2"
        onClick={() => {
          audio.play();
          setAudioPlayable(true);
        }}
      >
        Press to continue
      </Button>
    );
  }

  return (
    <Flex direction="column" gap="4">
      <Heading size="xl">Question {questionId + 1}</Heading>

      <Progress.Root
        value={Math.min(100, (audioCurrentTime / 10) * 100)}
        colorPalette="green"
      >
        <Progress.Track>
          <Progress.Range />
        </Progress.Track>
      </Progress.Root>

      <Flex direction="column" alignItems="center">
        {question.choices.map((choice, index) => (
          <Button
            key={index}
            type="button"
            onClick={() => handleChoiceSubmit(index)}
            disabled={selectedChoice !== null}
            height="auto"
            width="15em"
            wordWrap="break-word"
            whiteSpace="normal"
            backgroundColor={selectedChoice === index ? "blue" : "gray"}
            color="white"
            margin="1"
            padding="2"
          >
            {choice.name}
          </Button>
        ))}
      </Flex>

      <Scoreboard title="Scoreboard" users={users} />

      <Button
        onClick={() => {
          post(`/api/room/${room}/reset`, {});
        }}
      >
        Back to waiting room
      </Button>
    </Flex>
  );
};

export default Game;
