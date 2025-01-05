type SpotifyUser = {
  display_name: string;
};

type Playlist = {
  id: string;
  name: string;
  owner: SpotifyUser;
};

type Choice = {
  name: string;
};

type Question = {
  choices: Array<Choice>;
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
  EndedGameState,
};
