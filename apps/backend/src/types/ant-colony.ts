// Types for the Ant Colony Simulation Database Schema

export interface Database {
  public: {
    Tables: {
      simulations: {
        Row: {
          id: string
          name: string
          description: string | null
          world_width: number
          world_height: number
          created_at: string
          updated_at: string
          is_active: boolean
          simulation_speed: number
          current_tick: number
          season: string
          time_of_day: number
          weather_type: string
          weather_intensity: number
        }
        Insert: {
          id?: string
          name: string
          description?: string | null
          world_width?: number
          world_height?: number
          created_at?: string
          updated_at?: string
          is_active?: boolean
          simulation_speed?: number
          current_tick?: number
          season?: string
          time_of_day?: number
          weather_type?: string
          weather_intensity?: number
        }
        Update: {
          id?: string
          name?: string
          description?: string | null
          world_width?: number
          world_height?: number
          created_at?: string
          updated_at?: string
          is_active?: boolean
          simulation_speed?: number
          current_tick?: number
          season?: string
          time_of_day?: number
          weather_type?: string
          weather_intensity?: number
        }
      }
      ant_types: {
        Row: {
          id: number
          name: string
          base_speed: number
          base_strength: number
          base_health: number
          base_size: number
          lifespan_ticks: number
          carrying_capacity: number
          role: string
          color_hue: number
          special_abilities: any
          food_preferences: any
        }
        Insert: {
          id?: number
          name: string
          base_speed?: number
          base_strength?: number
          base_health?: number
          base_size?: number
          lifespan_ticks?: number
          carrying_capacity?: number
          role: string
          color_hue?: number
          special_abilities?: any
          food_preferences?: any
        }
        Update: {
          id?: number
          name?: string
          base_speed?: number
          base_strength?: number
          base_health?: number
          base_size?: number
          lifespan_ticks?: number
          carrying_capacity?: number
          role?: string
          color_hue?: number
          special_abilities?: any
          food_preferences?: any
        }
      }
      colonies: {
        Row: {
          id: string
          simulation_id: string
          name: string
          center_x: number
          center_y: number
          radius: number
          population: number
          color_hue: number
          resources: any
          nest_level: number
          territory_radius: number
          aggression_level: number
          created_at: string
          is_active: boolean
        }
        Insert: {
          id?: string
          simulation_id: string
          name: string
          center_x: number
          center_y: number
          radius?: number
          population?: number
          color_hue?: number
          resources?: any
          nest_level?: number
          territory_radius?: number
          aggression_level?: number
          created_at?: string
          is_active?: boolean
        }
        Update: {
          id?: string
          simulation_id?: string
          name?: string
          center_x?: number
          center_y?: number
          radius?: number
          population?: number
          color_hue?: number
          resources?: any
          nest_level?: number
          territory_radius?: number
          aggression_level?: number
          created_at?: string
          is_active?: boolean
        }
      }
      ants: {
        Row: {
          id: string
          colony_id: string
          ant_type_id: number
          position_x: number
          position_y: number
          angle: number
          current_speed: number
          health: number
          age_ticks: number
          state: string
          target_x: number | null
          target_y: number | null
          target_type: string | null
          target_id: string | null
          carried_resources: any
          traits: any
          energy: number
          mood: string
          last_updated: string
          created_at: string
        }
        Insert: {
          id?: string
          colony_id: string
          ant_type_id: number
          position_x: number
          position_y: number
          angle?: number
          current_speed: number
          health: number
          age_ticks?: number
          state?: string
          target_x?: number | null
          target_y?: number | null
          target_type?: string | null
          target_id?: string | null
          carried_resources?: any
          traits?: any
          energy?: number
          mood?: string
          last_updated?: string
          created_at?: string
        }
        Update: {
          id?: string
          colony_id?: string
          ant_type_id?: number
          position_x?: number
          position_y?: number
          angle?: number
          current_speed?: number
          health?: number
          age_ticks?: number
          state?: string
          target_x?: number | null
          target_y?: number | null
          target_type?: string | null
          target_id?: string | null
          carried_resources?: any
          traits?: any
          energy?: number
          mood?: string
          last_updated?: string
          created_at?: string
        }
      }
      food_sources: {
        Row: {
          id: string
          simulation_id: string
          food_type: string
          position_x: number
          position_y: number
          amount: number
          max_amount: number
          regeneration_rate: number
          discovery_difficulty: number
          nutritional_value: number
          spoilage_rate: number
          is_renewable: boolean
          created_at: string
        }
        Insert: {
          id?: string
          simulation_id: string
          food_type: string
          position_x: number
          position_y: number
          amount: number
          max_amount: number
          regeneration_rate?: number
          discovery_difficulty?: number
          nutritional_value?: number
          spoilage_rate?: number
          is_renewable?: boolean
          created_at?: string
        }
        Update: {
          id?: string
          simulation_id?: string
          food_type?: string
          position_x?: number
          position_y?: number
          amount?: number
          max_amount?: number
          regeneration_rate?: number
          discovery_difficulty?: number
          nutritional_value?: number
          spoilage_rate?: number
          is_renewable?: boolean
          created_at?: string
        }
      }
      pheromone_trails: {
        Row: {
          id: string
          colony_id: string
          trail_type: string
          position_x: number
          position_y: number
          strength: number
          decay_rate: number
          created_at: string
          expires_at: string | null
          source_ant_id: string | null
          target_food_id: string | null
        }
        Insert: {
          id?: string
          colony_id: string
          trail_type: string
          position_x: number
          position_y: number
          strength: number
          decay_rate?: number
          created_at?: string
          expires_at?: string | null
          source_ant_id?: string | null
          target_food_id?: string | null
        }
        Update: {
          id?: string
          colony_id?: string
          trail_type?: string
          position_x?: number
          position_y?: number
          strength?: number
          decay_rate?: number
          created_at?: string
          expires_at?: string | null
          source_ant_id?: string | null
          target_food_id?: string | null
        }
      }
    }
    Views: {}
    Functions: {}
    Enums: {}
  }
}

// Common types for ant colony simulation
export type Simulation = Database['public']['Tables']['simulations']['Row']
export type Colony = Database['public']['Tables']['colonies']['Row']
export type Ant = Database['public']['Tables']['ants']['Row']
export type AntType = Database['public']['Tables']['ant_types']['Row']
export type FoodSource = Database['public']['Tables']['food_sources']['Row']
export type PheromoneTrail = Database['public']['Tables']['pheromone_trails']['Row']

export interface AntState {
  WANDERING: 'wandering'
  SEEKING_FOOD: 'seeking_food'
  CARRYING_FOOD: 'carrying_food'
  FIGHTING: 'fighting'
  FLEEING: 'fleeing'
  DEAD: 'dead'
}

export interface PheromoneType {
  FOOD: 'food'
  DANGER: 'danger'
  TERRITORY: 'territory'
  RECRUITMENT: 'recruitment'
}

export interface FoodType {
  SEEDS: 'seeds'
  SUGAR: 'sugar'
  PROTEIN: 'protein'
  FRUIT: 'fruit'
}

export interface AntRole {
  WORKER: 'worker'
  SOLDIER: 'soldier'
  SCOUT: 'scout'
  QUEEN: 'queen'
  NURSE: 'nurse'
} 