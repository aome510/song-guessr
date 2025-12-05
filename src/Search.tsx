import { Playlist, QuestionType, User } from "./model.tsx";
import { useState } from "react";
import { get, put } from "./utils.tsx";
import {
  createListCollection,
  Input,
  Field,
  Select,
  Button,
  Flex,
} from "@chakra-ui/react";
import { Radio, RadioGroup } from "./components/ui/radio.tsx";
import LoadingPopup from "./components/LoadingPopup.tsx";

type NewGameRequest = {
  user_id: string;
  playlist_id: string;
  num_questions: number;
  question_types: Array<QuestionType>;
};

const numQuestionsChoices = createListCollection({
  items: Array.from({ length: 30 }, (_, i) => {
    return { value: i + 1 };
  }),
});

const questionTypeChoices = createListCollection({
  items: [
    { value: QuestionType.Song, label: "Song" },
    { value: QuestionType.Album, label: "Album" },
    { value: QuestionType.Artist, label: "Artist" },
  ],
});

const Search: React.FC<{ user: User; room: string }> = ({ room, user }) => {
  const [query, setQuery] = useState<string>("");
  const [results, setResults] = useState<Array<Playlist>>([]);
  const [numQuestions, setNumQuestions] = useState<number>(15);
  const [questionTypes, setQuestionTypes] = useState<Array<QuestionType>>([
    QuestionType.Song,
  ]);
  const [playlistId, setPlaylistId] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(false);

  const searchPlaylists = async () => {
    if (query !== "") {
      try {
        setLoading(true);
        const response = await get(`/api/search?query=${query}`);
        const data = await response.json();
        setLoading(false);

        setPlaylistId("");
        setResults(data);
        setNumQuestions(15);
      } catch (err) {
        console.error(err);
      }
    }
  };

  const newGame = async () => {
    const body: NewGameRequest = {
      user_id: user.id,
      playlist_id: playlistId,
      num_questions: numQuestions,
      question_types: questionTypes,
    };
    try {
      await put(`/api/room/${room}/new_game`, body);
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <div>
      <form
        onSubmit={async (e) => {
          e.preventDefault();
          setLoading(true);
          await newGame();
          setLoading(false);
        }}
      >
        <Flex gap="6" direction="column" maxW="800px" mx="auto" mt="6">
          <Field.Root>
        <Field.Label fontSize="lg" fontWeight="semibold">
          Search for a Spotify Playlist
        </Field.Label>
        <Flex gap="2">
          <Input
            type="text"
            placeholder="Enter playlist name..."
            size="lg"
            onChange={(e) => {
          setQuery(e.target.value);
            }}
            onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            searchPlaylists();
          }
            }}
          />
          <Button size="lg" onClick={searchPlaylists} colorScheme="blue">
            Search
          </Button>
        </Flex>
          </Field.Root>

          {results.length > 0 && (
        <Field.Root>
          <Field.Label fontSize="lg" fontWeight="semibold">
            Select a Playlist
          </Field.Label>
          <RadioGroup
            value={playlistId}
            onValueChange={(e) => {
          setPlaylistId(e.value);
            }}
          >
            <Flex
          direction="column"
          gap="2"
          p="4"
          borderWidth="1px"
          borderRadius="md"
          maxH="300px"
          overflowY="auto"
            >
          {results.slice(0, 10).map((result) => (
            <Radio
              key={result.id}
              value={result.id}
              p="2"
              borderRadius="md"
              _hover={{ bg: "gray.50" }}
            >
              <Flex direction="column">
            <span style={{ fontWeight: "500" }}>{result.name}</span>
            <span style={{ fontSize: "0.9em", color: "gray" }}>
              by {result.owner.display_name}
            </span>
              </Flex>
            </Radio>
          ))}
            </Flex>
          </RadioGroup>
        </Field.Root>
          )}

          {results.length > 0 && (
        <Flex gap="4" direction={{ base: "column", md: "row" }}>
          <Field.Root flex="1">
            <Select.Root
          collection={numQuestionsChoices}
          // @ts-expect-error: value of Select component is array of numbers
          value={[numQuestions]}
          onValueChange={(e) => {
            setNumQuestions(e.items[0].value);
          }}
            >
          <Select.Label fontWeight="semibold">
            Number of Questions
          </Select.Label>
          <Select.Trigger>
            <Select.ValueText />
          </Select.Trigger>
          <Select.Content>
            {numQuestionsChoices.items.map((item) => (
              <Select.Item color="black" item={item} key={item.value}>
            {item.value}
              </Select.Item>
            ))}
          </Select.Content>
            </Select.Root>
          </Field.Root>

          <Field.Root flex="1">
            <Select.Root
          multiple
          collection={questionTypeChoices}
          value={questionTypes}
          onValueChange={(e) => {
            setQuestionTypes(e.items.map((item) => item.value));
          }}
            >
          <Select.Label fontWeight="semibold">
            Question Types
          </Select.Label>
          <Select.Trigger>
            <Select.ValueText />
          </Select.Trigger>
          <Select.Content>
            {questionTypeChoices.items.map((item) => (
              <Select.Item color="black" item={item} key={item.value}>
            {item.label}
              </Select.Item>
            ))}
          </Select.Content>
            </Select.Root>
          </Field.Root>
        </Flex>
          )}

          {results.length > 0 && (
        <Button
          type="submit"
          size="lg"
          colorScheme="green"
          disabled={playlistId === "" || questionTypes.length == 0}
        >
          Start new game
        </Button>
          )}
        </Flex>
      </form>

      <LoadingPopup loading={loading} />
    </div>
  );
};

export default Search;
