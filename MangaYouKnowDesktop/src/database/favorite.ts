import Database from '@tauri-apps/plugin-sql';
import type { Favorite, Mark } from '~/models';


export async function createFavorite(favorite: Favorite, userId?: number): Promise<void> {
	if (userId === undefined) {
		return
	}
	const db = await Database.load('sqlite:data.db');
	try {
		await db.execute(
			'INSERT INTO favorite (user_id, name, folder_name, cover, source, source_id, type, extra_name, title_color, card_color, grade, author, description) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)',
			[
				userId,
				favorite.name,
				favorite.folder_name,
				favorite.cover,
				favorite.source,
				favorite.source_id,
				favorite.type,
				favorite.extra_name,
				favorite.title_color,
				favorite.card_color,
				favorite.grade,
				favorite.author,
				favorite.description
			]
		);
	} catch (error) {
		console.log(error)
	} finally {
		// db.close()
	}
}
export async function getFavorites(userID: number | undefined, query: string = ''): Promise<Favorite[]> {
	const db = await Database.load('sqlite:data.db');
	try {
		if (query === '') {
			const favorites: Favorite[] = await db.select(
				'SELECT * FROM favorite WHERE user_id = ?', 
				[userID]
			);
			return favorites
		} else {
			const favorites: Favorite[] = await db.select(
				'SELECT * FROM favorite WHERE user_id = ? AND INSTR(LOWER(NAME), ?) > 0', 
				[userID, query.toLowerCase()]
			);
			return favorites
		}
	} catch (error) {
		console.log(error)
		return [] 
	} finally {
		// db.close()
	}
}

export async function getUltraFavorites(userID: number | undefined): Promise<Favorite[]> {
	const db = await Database.load('sqlite:data.db');
	try {
		const favorites: Favorite[] = await db.select(
			'SELECT * FROM favorite WHERE user_id = ? AND is_ultra_favorite = ?', 
			[userID, true]
		);
		return favorites
	} catch (error) {
		console.log(error)
		return [] 
	} finally {
		// db.close()
	}
}

export async function getFavoritesByMark(userID: number | undefined, mark: Mark): Promise<Favorite[]> {
	const db = await Database.load('sqlite:data.db');
	try {
		const favorites: Favorite[] = await db.select(
			'SELECT * FROM favorite WHERE user_id = ? AND id IN (SELECT favorite_id FROM mark_favorites WHERE mark_id = ?)',
			[userID, mark.id]
		);
		return favorites
	} catch (error) {
		console.log(error)
		return [] 
	} finally {
		// db.close()
	}
}

export async function getFavoritesByTypes(userID: number | undefined, types: string[]): Promise<Favorite[]> {
	const db = await Database.load('sqlite:data.db');
	try {
		const favorites: Favorite[] = await db.select(
			'SELECT * FROM favorite WHERE user_id = ? AND type in (?)',
			[userID, types]
		);
		return favorites
	} catch (error) {
		console.log(error)
		return [] 
	} finally {
		// db.close()
	}
}

export async function updateFavorite(favorite: Favorite): Promise<void> {
	const db = await Database.load('sqlite:data.db');
	try {
		await db.execute(
			'UPDATE favorite SET name = ?, folder_name = ?, cover = ?, source = ?, source_id = ?, type = ?, extra_name = ?, title_color = ?, card_color = ?, grade = ?, author = ?, description = ?, is_ultra_favorite = ? WHERE id = ?',
			[
				favorite.name,
				favorite.folder_name,
				favorite.cover,
				favorite.source,
				favorite.source_id,
				favorite.type,
				favorite.extra_name,
				favorite.title_color,
				favorite.card_color,
				favorite.grade,
				favorite.author,
				favorite.description,
				favorite.is_ultra_favorite,
				favorite.id
			]
		);
	} catch (error) {
		console.log(error)
	} finally {
		// db.close()
	}
}

export async function deleteFavorite(favorite: Favorite): Promise<void> {
	const db = await Database.load('sqlite:data.db');
	try {
		await db.execute(
			'DELETE FROM favorite WHERE id = ?',
			[favorite.id]
		);
		await db.execute(
			'DELETE FROM readed WHERE favorite_id = ?',
			[favorite.id]
		);
	} catch (error) {
		console.log(error)
	} finally {
		// db.close()
	}
}
