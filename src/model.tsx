type User = {
  display_name: string;
};

type Playlist = {
  id: string;
  name: string;
  owner: User;
};

type Choice = {
  name: string;
  preview_url: string;
};

type Question = {
  choices: Array<Choice>;
  ans_id: number;
};

export type { User, Playlist, Choice, Question };
