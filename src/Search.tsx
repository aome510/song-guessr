import { Playlist } from "./model.tsx";
import { useState } from "react";
import { get, post } from "./utils.tsx";
import {
  createListCollection,
  Heading,
  Input,
  Field,
  Select,
  Button,
} from "@chakra-ui/react";

type NewGameRequest = {
  playlist_id: string;
  num_questions: number;
};

const numQuestionsChoices = createListCollection({
  items: Array.from({ length: 30 }, (_, i) => {
    return { value: i + 1 };
  }),
});

const Search: React.FC<{ room: string }> = ({ room }) => {
  const [query, setQuery] = useState<string>("");
  const [results, setResults] = useState<Array<Playlist>>([]);
  const [numQuestions, setNumQuestions] = useState<number>(15);
  const [playlistId, setPlaylistId] = useState<string>("");

  const searchPlaylists = async () => {
    if (query !== "") {
      try {
        const response = await get(`/api/search?query=${query}`);
        const data = await response.json();
        console.assert(data instanceof Array, "Expected an array of playlists");
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
    };
    try {
      await post(`/api/room/${room}/game`, body);
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <div>
      <Heading size="4xl">Search for playlist</Heading>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          newGame();
        }}
      >
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
        {results.slice(0, 10).map((result) => (
          <div key={result.id}>
            <label>
              <input
                type="radio"
                name="radioGroup"
                checked={playlistId === result.id}
                onChange={() => setPlaylistId(result.id)}
              />
              {result.name} by {result.owner.display_name}
            </label>
          </div>
        ))}
        {results.length > 0 && (
          <div>
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
            <Button type="submit" disabled={playlistId === ""}>
              Submit
            </Button>
          </div>
        )}
      </form>
    </div>
  );
};

export default Search;
