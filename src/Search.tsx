import { Playlist, QuestionType } from "./model.tsx";
import { useState } from "react";
import { get, post } from "./utils.tsx";
import {
  createListCollection,
  Heading,
  Input,
  Field,
  Select,
  Button,
  Flex,
} from "@chakra-ui/react";
import { Radio, RadioGroup } from "./components/ui/radio.tsx";
import LoadingPopup from "./components/LoadingPopup.tsx";

type NewGameRequest = {
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

const Search: React.FC<{ room: string }> = ({ room }) => {
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
      playlist_id: playlistId,
      num_questions: numQuestions,
      question_types: questionTypes,
    };
    try {
      await post(`/api/room/${room}/game`, body);
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <div>
      <Heading size="xl">Search for playlist</Heading>

      <form
        onSubmit={async (e) => {
          e.preventDefault();
          setLoading(true);
          await newGame();
          setLoading(false);
        }}
      >
        <Flex gap="3" direction="column">
          <Field.Root>
            <Field.Label>Search</Field.Label>
            <Input
              type="text"
              onChange={(e) => {
                setQuery(e.target.value);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault(); // prevent form submission on Enter
                  searchPlaylists();
                }
              }}
            />
          </Field.Root>

          <RadioGroup
            value={playlistId}
            onValueChange={(e) => {
              setPlaylistId(e.value);
            }}
          >
            <Flex direction="column" gap="1">
              {results.slice(0, 10).map((result) => (
                <Radio key={result.id} value={result.id}>
                  {result.name} by {result.owner.display_name}
                </Radio>
              ))}
            </Flex>
          </RadioGroup>

          {results.length > 0 && (
            <Select.Root
              collection={numQuestionsChoices}
              // @ts-expect-error: value of Select component is array of numbers
              value={[numQuestions]}
              onValueChange={(e) => {
                setNumQuestions(e.items[0].value);
              }}
            >
              <Select.Label>Number of questions</Select.Label>
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
          )}

          {results.length > 0 && (
            <Select.Root
              multiple
              collection={questionTypeChoices}
              value={questionTypes}
              onValueChange={(e) => {
                setQuestionTypes(e.items.map((item) => item.value));
              }}
            >
              <Select.Label>Question types</Select.Label>
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
          )}

          {results.length > 0 && (
            <Button
              type="submit"
              disabled={playlistId === "" || questionTypes.length == 0}
            >
              New Game
            </Button>
          )}
        </Flex>
      </form>

      <LoadingPopup loading={loading} />
    </div>
  );
};

export default Search;
