import { IsString, IsNotEmpty } from 'class-validator';

export interface JwtPayload {
  sub: string;
  username: string;
  role: string;
  permissions: string[];
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
  @IsString()
  @IsNotEmpty()
  username: string;

  @IsString()
  @IsNotEmpty()
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