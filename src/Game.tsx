import { useEffect, useMemo, useState } from "react";
import { PlayingGameState, User } from "./model.tsx";
import { Button, Flex, Progress, Text } from "@chakra-ui/react";
import { put } from "./utils.tsx";
import Scoreboard from "./components/Scoreboard.tsx";
import { Howl } from "howler";

const Game: React.FC<{
  ws: WebSocket;
  state: PlayingGameState;
  user: User;
  room: string;
  isOwner: boolean;
}> = ({ ws, state, user, room, isOwner }) => {
  const [selectedChoice, setSelectedChoice] = useState<number | null>(null);
  const [audioCurrentTime, setAudioCurrentTime] = useState<number>(0);
  const [audioPlayable, setAudioPlayable] = useState<boolean>(true);
  // construct a timer to measure the elapsed time of the current song's progress
  const [timer] = useState(performance.now() - state.song_progress_ms);

  const audio = useMemo(() => {
    const audio = new Howl({
      src: [state.question.song_url],
      format: ["mp3"],
      html5: true,
      onplayerror: () => {
        setAudioPlayable(false);
      },
      autoplay: true,
      volume: 0.5,
    });

    audio.on("play", () => {
      setAudioPlayable(true);
      const progress = (performance.now() - timer) / 1000;
      audio.seek(progress);
    });

    return audio;
  }, [state.question.song_url, timer]);

  useEffect(() => {
    const interval = setInterval(() => {
      setAudioCurrentTime(audio.seek());
    }, 100);

    return () => {
      audio.pause();
      clearInterval(interval);
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
        submitted_at_ms: Math.round(audio.seek() * 1000),
      }),
    );
  };

  // this is a hack to get the audio to play on the first render
  // because the audio autoplay must be triggered by a user gesture
  // more details: see https://developer.chrome.com/blog/autoplay/
  if (audioPlayable === false) {
    return (
      <Button
        padding="2"
        onClick={() => {
          audio.play();
        }}
      >
        Press to continue
      </Button>
    );
  }

  return (
    <Flex direction="column" gap="4">
      <Text textStyle="xl" fontWeight="semibold">
        Question {state.question_id + 1}
      </Text>
      <Text textStyle="md">
        Score:&nbsp;
        <Text textStyle="lg" color="green.500" as="span">
          {state.question.score}
        </Text>
        , fastest bonus:&nbsp;
        <Text textStyle="lg" color="green.500" as="span">
          {state.question.bonus}
        </Text>
      </Text>

      {audio.playing() && (
        <Progress.Root
          value={Math.min(100, (audioCurrentTime / 10) * 100)}
          colorPalette="green"
        >
          <Progress.Track>
            <Progress.Range />
          </Progress.Track>
        </Progress.Root>
      )}

      <Flex direction="column" alignItems="center">
        <Text textStyle="lg" fontWeight="bold">
          Guess the {state.question.question_type}
        </Text>
        {state.question.choices.map((choice, index) => (
          <Button
            key={index}
            type="button"
            onClick={() => handleChoiceSubmit(index)}
            disabled={selectedChoice !== null || !audio.playing()}
            height="auto"
            width="15em"
            wordWrap="break-word"
            whiteSpace="normal"
            backgroundColor={selectedChoice === index ? "blue" : "gray"}
            color="white"
            margin="1"
            padding="2"
          >
            {choice}
          </Button>
        ))}
      </Flex>

      <Scoreboard title="Scoreboard" users={state.users} />

      {isOwner && (
        <Button
          onClick={() => {
            put(`/api/room/${room}/reset`, { user_id: user.id });
          }}
        >
          Back to Lobby
        </Button>
      )}
    </Flex>
  );
};

export default Game;
