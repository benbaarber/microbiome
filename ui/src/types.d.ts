export type Entity = {
  pos: [number, number];
  mass: number;
  radius: number;
  color: string;
  [key: string]: any;
};

export type FrameData = {
  agent: Entity;
  npcs: Entity[];
  food: Entity[];
};
