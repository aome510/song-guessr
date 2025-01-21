import { useEffect, useMemo, useState } from "react";
import { PlayingGameState, User } from "./model.tsx";
import { Button, Flex, Progress, Text } from "@chakra-ui/react";
import { post } from "./utils.tsx";
import Scoreboard from "./components/Scoreboard.tsx";
import { Howl } from "howler";

const Game: React.FC<{
  ws: WebSocket;
  state: PlayingGameState;
  user: User;
  room: string;
}> = ({ ws, state, user, room }) => {
  const [selectedChoice, setSelectedChoice] = useState<number | null>(null);
  const [audioCurrentTime, setAudioCurrentTime] = useState<number>(0);
  const [audioPlayable, setAudioPlayable] = useState<boolean>(true);

  const audio = useMemo(() => {
    return new Howl({
      src: [state.question.song_url],
      format: ["mp3"],
      html5: true,
      onplayerror: () => {
        setAudioPlayable(false);
      },
      onplay: () => {
        setAudioPlayable(true);
      },
      autoplay: true,
      volume: 0.5,
    });
  }, [state.question.song_url]);

  useEffect(() => {
    const gap = Math.abs(state.song_progress_ms / 1000 - audio.seek());
    // gap > 1.0 indicates the player joining late.
    // We disallow the player from playing this round (by pausing the audio)
    // because the audio can be out of sync with the server
    // TODO: improve this by syncing the audio progress with the server
    if (gap > 1.0) {
      audio.pause();
    }
  }, [audio, state.song_progress_ms]);

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

      <Button
        onClick={() => {
          post(`/api/room/${room}/reset`, {});
        }}
      >
        Back to Lobby
      </Button>
    </Flex>
  );
};

export default Game;
