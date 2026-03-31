import { betterAuth } from 'better-auth'
import { passkey } from '@better-auth/passkey'
import { drizzleAdapter } from 'better-auth/adapters/drizzle'
import { tanstackStartCookies } from 'better-auth/tanstack-start'
import { postgres_db } from '@ant-colony-simulator/db-drizzle'

const baseUrl = process.env.BETTER_AUTH_URL || 'http://localhost:3000'

export const auth = betterAuth({
  baseURL: baseUrl,
  secret: process.env.BETTER_AUTH_SECRET,
  database: drizzleAdapter(postgres_db, {
    provider: 'pg',
  }),
  emailAndPassword: {
    enabled: true,
  },
  plugins: [
    passkey({
      rpID: process.env.PASSKEY_RP_ID || 'localhost',
      rpName: 'Ant Colony Simulator',
      origin: process.env.PASSKEY_ORIGIN || 'http://localhost:3000',
    }),
    tanstackStartCookies(),
  ],
})
