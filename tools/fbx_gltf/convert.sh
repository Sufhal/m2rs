#!/bin/bash

# Répertoire source contenant les fichiers FBX
SOURCE_DIR=../../assets/pack/zone
# Répertoire cible où les fichiers GLB seront enregistrés
TARGET_DIR=../../assets/pack/zone

# Crée le répertoire cible s'il n'existe pas encore
mkdir -p "$TARGET_DIR"

# Parcours récursivement tous les fichiers .fbx dans le répertoire source
find "$SOURCE_DIR" -type f -name "*.fbx" | while read fbx_file; do
    # Nom de fichier sans l'extension
    base_name=$(basename "$fbx_file" .fbx)
    
    # Répertoire de sortie = répertoire du fichier d'entrée
    output_dir=$(dirname "$fbx_file")
    
    # Chemin de sortie du fichier .glb correspondant
    output_file="$output_dir/$base_name.glb"
    
    # Exécute la commande FBX2glTF pour convertir le fichier
    ./fbx_gltf --binary \
        --input "$fbx_file" \
        --output "$output_file"

    echo "Converti: $fbx_file -> $output_file"
done