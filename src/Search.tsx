import { Playlist } from "./model.tsx";
import { useState } from "react";
import { makeApiRequest } from "./utils.tsx";
import { useNavigate } from "react-router-dom";

function Search() {
  const [query, setQuery] = useState<string>("");
  const [results, setResults] = useState<Array<Playlist>>([]);
  const [numQuestions, setNumQuestions] = useState<number>(15);
  const [playlistId, setPlaylistId] = useState<string>("");
  const navigate = useNavigate();

  const searchPlaylists = async () => {
    if (query !== "") {
      try {
        const response = await makeApiRequest(
          `http://localhost:8000/search/${query}`,
        );
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

  return (
    <div>
      <h2>Search for playlist</h2>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          navigate(
            `/game?playlist_id=${playlistId}&num_questions=${numQuestions}`,
          );
        }}
      >
        <label>
          Query:
          <input
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
        </label>
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
            <div>
              <label>
                Number of Questions:
                <select
                  value={numQuestions}
                  onChange={(e) => setNumQuestions(Number(e.target.value))}
                >
                  {Array.from({ length: 30 }, (_, i) => i + 1).map((num) => (
                    <option key={num} value={num}>
                      {num}
                    </option>
                  ))}
                </select>
              </label>
            </div>
            <div>
              <button type="submit" disabled={playlistId === ""}>
                Submit
              </button>
            </div>
          </div>
        )}
      </form>
    </div>
  );
}

export default Search;
