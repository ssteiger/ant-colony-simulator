# Ant Colony Simulator

## Local development

```bash
# install dependencies
npm install

# create .env file
cp apps/backend/.env.example apps/backend/.env
cp apps/frontend/.env.example apps/frontend/.env
cp packages/db-drizzle/.env.example packages/db-drizzle/.env

# start all services
turbo dev

# copy the SUPABASE_ANON_KEY from the console into apps/web/.env and apps/frontend/.env
```

### Start single Services

```bash
# run local supabase server
npm run dev:db

# copy the SUPABASE_ANON_KEY from the console into apps/web/.env and apps/frontend/.env

# open supabase dashboard at http://127.0.0.1:54323/project/default
```

```bash
# run backend
npm run dev:backend

# run web app
npm run dev:frontend

# open web app at http://127.0.0.1:3000
```

## Helpers

### Generate types

```bash
cd packages/db-drizzle
npx drizzle-kit generate
```

### Reset database

```bash
cd apps/supabase
npx supabase db reset
```

## Prep for first time setup

```bash
# install turbo cli
npm install turbo --global
```
