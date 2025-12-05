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
  const [progress, setProgress] = useState<number>(0);
  // construct a timer to measure the elapsed time of the current song's progress
  const [timer] = useState(performance.now() - state.song_progress_ms);

  const audio = useMemo(() => {
    const audio = new Howl({
      src: [state.question.song_url],
      format: ["mp3"],
      html5: true,
      autoplay: true,
      volume: 0.5,
    });

    audio.play();

    audio.on("play", () => {
      audio.seek((performance.now() - timer) / 1000);
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

  useEffect(() => {
    const interval = setInterval(() => {
      setProgress((performance.now() - timer) / 1000);
    }, 100);

    return () => {
      clearInterval(interval);
    };
  }, [timer]);

  const handleChoiceSubmit = (selectedChoice: number) => {
    setSelectedChoice(selectedChoice);
    ws.send(
      JSON.stringify({
        type: "UserSubmitted",
        user_name: user.name,
        user_id: user.id,
        selected_id: selectedChoice,
        submitted_at_ms: Math.round(audio.seek() * 1000),
      }),
    );
  };

  // this is a hack to get the audio to play on the first render
  // because the audio autoplay must be triggered by a user gesture
  // more details: see https://developer.chrome.com/blog/autoplay/
  if (!audio.playing() && progress > 1.0) {
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
    <Flex direction="column" gap="6" width="100%" maxW="600px" mx="auto">
      <Flex direction="column" gap="2" align="center">
      <Text textStyle="2xl" fontWeight="bold" color="gray.700">
        Question {state.question_id + 1}
      </Text>
      <Flex gap="4" wrap="wrap" justify="center">
        <Flex align="center" gap="1">
        <Text textStyle="sm" color="gray.600">Score:</Text>
        <Text textStyle="xl" color="green.500" fontWeight="bold">
          {state.question.score}
        </Text>
        </Flex>
        <Flex align="center" gap="1">
        <Text textStyle="sm" color="gray.600">Fastest Bonus:</Text>
        <Text textStyle="xl" color="orange.500" fontWeight="bold">
          {state.question.bonus}
        </Text>
        </Flex>
      </Flex>
      </Flex>

      {audio.playing() && (
      <Progress.Root
        value={Math.min(100, (audioCurrentTime / 10) * 100)}
        colorPalette="green"
        size="lg"
      >
        <Progress.Track bg="gray.200" borderRadius="full">
        <Progress.Range borderRadius="full" />
        </Progress.Track>
      </Progress.Root>
      )}

      <Flex direction="column" gap="4" p="6" bg="gray.50" borderRadius="xl" shadow="md">
      <Text textAlign="center" textStyle="xl" fontWeight="bold" color="gray.700">
        Guess the {state.question.question_type}
      </Text>
      <Flex direction="column" gap="3">
        {state.question.choices.map((choice, index) => (
        <Button
          key={index}
          type="button"
          onClick={() => handleChoiceSubmit(index)}
          disabled={selectedChoice !== null || !audio.playing()}
          height="auto"
          minH="60px"
          width="100%"
          fontSize="lg"
          fontWeight="medium"
          whiteSpace="normal"
          textAlign="center"
          px="4"
          py="3"
          borderRadius="lg"
          transition="all 0.2s"
          backgroundColor={
          selectedChoice === index 
            ? "blue.500" 
            : "white"
          }
          color={selectedChoice === index ? "white" : "gray.700"}
          border="2px solid"
          borderColor={selectedChoice === index ? "blue.500" : "gray.300"}
          _hover={{
          transform: selectedChoice === null && audio.playing() ? "translateY(-2px)" : "none",
          shadow: selectedChoice === null && audio.playing() ? "lg" : "none",
          borderColor: selectedChoice === null && audio.playing() ? "blue.400" : undefined,
          }}
          _disabled={{
          opacity: selectedChoice === null ? 0.5 : 1,
          cursor: "not-allowed",
          }}
        >
          {choice}
        </Button>
        ))}
      </Flex>
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
