import { useRef, useState } from "react";
import "./App.css";

type User = {
  display_name: string;
};

type Playlist = {
  id: string;
  name: string;
  owner: User;
};

function App() {
  const audioRef = useRef(new Audio());
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Array<Playlist>>([]);
  const [playlistId, setPlaylistId] = useState("");

  const makeApiRequest = async (url: string): Promise<Response> => {
    const response = await fetch(url);
    if (response.status !== 200) {
      const text = await response.text();
      throw new Error(`Failed to make API request ${url}: ${text}`);
    } else {
      return response;
    }
  };

  const fetchAudioUrl = async () => {
    if (playlistId !== "") {
      try {
        const response = await makeApiRequest(
          `http://localhost:8000/questions/${playlistId}`,
        );
        audioRef.current.src = await response.text();
      } catch (err) {
        console.error(err);
      }
    }
  };

  const searchPlaylists = async () => {
    if (query !== "") {
      try {
        const response = await makeApiRequest(
          `http://localhost:8000/search/${query}`,
        );
        const data = await response.json();
        console.assert(data instanceof Array, "Expected an array of playlists");
        setResults(data);
      } catch (err) {
        console.error(err);
      }
    }
  };

  return (
    <>
      <h1>Song Guesser</h1>
      <div>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            fetchAudioUrl();
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
          <button type="submit">Submit</button>
        </form>
      </div>
      <div>
        <audio ref={audioRef} autoPlay controls />
      </div>
      <div>
        <button onClick={fetchAudioUrl}>Re-generate</button>
      </div>
    </>
  );
}

export default App;
