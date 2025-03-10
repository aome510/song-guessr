type SpotifyUser = {
  display_name: string;
};

type Playlist = {
  id: string;
  name: string;
  owner: SpotifyUser;
};

enum QuestionType {
  Song = "Song",
  Artist = "Artist",
  Album = "Album",
}

type Question = {
  question_type: QuestionType;
  choices: Array<string>;
  score: number;
  bonus: number;
  song_url: string;
};

type User = {
  id: string;
  name: string;
};

type UserGameState = {
  name: string;
  score: number;
};

type PlayingGameState = {
  question: Question;
  question_id: number;
  song_progress_ms: number;
  users: Array<UserGameState>;
};

type UserSubmission = {
  user_name: string;
  score: number;
  submitted_at_ms: number;
};

type WaitingForNextQuestionState = {
  answer: string;
  correct_submissions: Array<UserSubmission>;
  users: Array<UserGameState>;
};

type WaitingGameState = {
  users: Array<UserGameState>;
};

type EndedGameState = {
  users: Array<UserGameState>;
};

export type {
  Playlist,
  User,
  UserGameState,
  Question,
  PlayingGameState,
  WaitingGameState,
  WaitingForNextQuestionState,
  EndedGameState,
  UserSubmission,
};

export { QuestionType };
