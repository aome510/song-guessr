import { useEffect, useState } from "react";
import { Question } from "./model.tsx";
import { makeApiRequest } from "./utils.tsx";

function Game() {
  const [selectedChoice, setSelectedChoice] = useState<number>(0);
  const [questionId, setQuestionId] = useState<number>(-1);
  const [questions, setQuestions] = useState<Array<Question>>([]);
  const [audioSrc, setAudioSrc] = useState<string>("");
  const queryParams = new URLSearchParams(window.location.search);
  const playlistId = queryParams.get("playlist_id");
  const numQuestions = queryParams.get("num_questions") || "15";

  useEffect(() => {
    const fetchQuestions = async () => {
      if (playlistId !== "") {
        try {
          const response = await makeApiRequest(
            `http://localhost:8000/questions/${playlistId}?num_questions=${numQuestions}`,
          );
          const data = await response.json();
          console.assert(
            data instanceof Array,
            "Expected an array of questions",
          );
          setQuestions(data);
          setQuestionId(0);
          setAudioSrc(data[0].choices[data[0].ans_id].preview_url);
        } catch (err) {
          console.error(err);
        }
      }
    };

    fetchQuestions();
  }, [numQuestions, playlistId]);

  const handleChoiceSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    const question = questions[questionId];
    if (selectedChoice === question.ans_id) {
      alert("Correct!");
    } else {
      alert(
        "Incorrect! The correct song is " +
          question.choices[question.ans_id].name,
      );
    }
    // Move to the next question
    if (questionId + 1 < questions.length) {
      setQuestionId(questionId + 1);
      setSelectedChoice(0); // Reset selected choice for the next question
      const next_question = questions[questionId + 1];
      setAudioSrc(next_question.choices[next_question.ans_id].preview_url);
    } else {
      alert("Quiz completed!");
    }
  };

  if (questionId === -1) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <h2>Question {questionId + 1}</h2>
      <audio src={audioSrc} autoPlay controls />
      <form onSubmit={handleChoiceSubmit}>
        {questions[questionId].choices.map((choice, index) => (
          <button
            key={choice.name}
            type="button"
            onClick={() => setSelectedChoice(index)}
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
        <div>
          <button type="submit">Submit</button>
        </div>
      </form>
    </div>
  );
}

export default Game;
