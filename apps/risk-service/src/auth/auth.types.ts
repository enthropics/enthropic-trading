export interface JwtPayload {
  sub: string;
  username: string;
  role: string;
  permissions: string[];
  exp: number;
  iat: number;
  jti: string;
}

export interface AuthenticatedUser {
  accountId: string;
  username: string;
  role: string;
  permissions: Set<string>;
  tokenJti: string;
}

export class LoginDto {
  username: string;
  password: string;
}

export class TokenResponseDto {
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
  user: {
    id: string;
    username: string;
    role: string;
    permissions: string[];
  };
}
