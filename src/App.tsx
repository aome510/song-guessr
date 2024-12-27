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
  const [selectedChoice, setSelectedChoice] = useState(0);
  const [questionId, setQuestionId] = useState(0);
  const [questions, setQuestions] = useState([]);

  const makeApiRequest = async (url: string): Promise<Response> => {
    const response = await fetch(url);
    if (response.status !== 200) {
      const text = await response.text();
      throw new Error(`Failed to make API request ${url}: ${text}`);
    } else {
      return response;
    }
  };

  const fetchQuestions = async () => {
    if (playlistId !== "") {
      try {
        const response = await makeApiRequest(
          `http://localhost:8000/questions/${playlistId}`,
        );
        const data = await response.json();
        console.assert(data instanceof Array, "Expected an array of questions");
        setQuestions(data);
        setQuestionId(0);
        audioRef.current.src = data[0].choices[data[0].ans_id].preview_url;
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

  const handleChoiceClick = (id: number) => {
    setSelectedChoice(id);
  };

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    if (selectedChoice === questions[questionId].ans_id) {
      alert("Correct!");
    } else {
      alert(
        "Incorrect! Correct song is: " +
          questions[questionId].choices[questions[questionId].ans_id].name,
      );
    }
    // Move to the next question
    if (questionId < questions.length - 1) {
      setQuestionId(questionId + 1);
      setSelectedChoice(0); // Reset selected choice for the next question
      audioRef.current.src =
        questions[questionId + 1].choices[
          questions[questionId + 1].ans_id
        ].preview_url;
    } else {
      alert("Quiz completed!");
    }
  };

  return (
    <>
      <h1>Song Guessr</h1>
      <div>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            fetchQuestions();
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
      {questions.length > 0 && (
        <div>
          <h2>Question: {questionId + 1}</h2>
          <form onSubmit={handleSubmit}>
            {questions[questionId].choices.map((choice, index) => (
              <button
                key={choice.name}
                type="button"
                onClick={() => handleChoiceClick(index)}
                style={{
                  backgroundColor: selectedChoice === index ? "blue" : "gray",
                  color: "white",
                  margin: "5px",
                  padding: "10px",
                }}
              >
                {choice.name}
              </button>
            ))}
            <button type="submit">Submit</button>
          </form>
        </div>
      )}
      <div></div>
    </>
  );
}

export default App;
