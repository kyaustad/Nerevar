export type Tes3MPServerConfig = {
  general: {
    local_address: string;
    port: number;
    maximum_players: number;
    hostname: string;
    log_level: 0 | 1 | 2 | 3 | 4;
    password: string;
  };
  plugins: {
    home: string;
    plugins: string;
  };
  master_server: {
    enabled: boolean;
    address: string;
    port: number;
    rate: number;
  };
};
